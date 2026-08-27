use crate::agent_runtime;
use crate::edge_policy::{AutoPriority, EdgeDeviceProfile, decide_quant_mode};
use crate::model::{AnyKvCache, NativeGemma, QuantMode, QuantizedGemma};
use crate::quant::QuantizedActivationMode;
use crate::qwen35::{self, Qwen35TextModel};
use crate::sampling::SamplingConfig;
use crate::scheduler::{PrefixValue, RuntimeScheduler};
use crate::speculative::{AdaptiveDraftController, FastNGramProposalEngine};
use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
    routing::{get, post},
};
use rand::{SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokenizers::Tokenizer;

const REQUEST_QUEUE_CAPACITY: usize = 128;
const RESPONSE_QUEUE_CAPACITY: usize = 32;
const DEFAULT_PREFILL_CHUNK_TOKENS: usize = 32;
const MAX_REQUEST_BODY_BYTES: usize = 1_048_576;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WorkerMessage {
    Delta {
        text: String,
    },
    Done {
        text: String,
        prompt_tokens: usize,
        completion_tokens: usize,
        finish_reason: String,
        plan: ZymaticaPlan,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Default, Clone)]
struct RequestMetrics {
    queue_time_ms: u64,
    prefill_time_ms: u64,
    token_latencies_ms: Vec<u64>,
}

struct QueueItem {
    request_id: u64,
    prompt_ids: Vec<usize>,
    max_tokens: usize,
    temperature: f32,
    top_k: usize,
    top_p: Option<f32>,
    min_p: Option<f32>,
    seed: u64,
    stop_sequences: Vec<String>,
    response_tx: tokio::sync::mpsc::Sender<Result<String, String>>,
}

struct QueueRequest {
    prompt_ids: Vec<usize>,
    max_tokens: usize,
    temperature: f32,
    top_k: usize,
    top_p: Option<f32>,
    min_p: Option<f32>,
    seed: u64,
    stop_sequences: Vec<String>,
}

struct ActiveRequest {
    id: u64,
    prompt_ids: Vec<usize>,
    generated_ids: Vec<usize>,
    max_new_tokens: usize,
    sampling: SamplingConfig,
    rng: rand::rngs::StdRng,
    response_tx: tokio::sync::mpsc::Sender<Result<String, String>>,
    created_time: Instant,
    last_logits: Vec<f32>,
    draft_cache: Option<AnyKvCache>,
    draft_last_logits: Vec<f32>,
    draft_prefill_pos: usize,
    draft_controller: Option<AdaptiveDraftController>,
    ngram_proposer: Option<FastNGramProposalEngine>,
    prefill_done: bool,
    prefill_pos: usize,
    last_touched_iteration: u64,
    kv_swapped_path: Option<PathBuf>,
    kv_swap_count: u64,
    decoded_len: usize,
    stop_sequences: Vec<String>,
    reusable_prefix_tokens: usize,
    metrics: RequestMetrics,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: String,
    pub model_dir: PathBuf,
    pub tokenizer: PathBuf,
    pub engine: String,
    pub q8_cache_dir: Option<PathBuf>,
    pub max_new_tokens: usize,
    pub scheduler_max_batch_tokens: usize,
    pub prefill_chunk_tokens: usize,
    pub kv_swap_dir: Option<PathBuf>,
    pub kv_max_resident_pages: usize,
    pub kv_swap_threshold: f32,
    pub draft_model_dir: Option<PathBuf>,
    pub draft_engine: String,
    pub draft_cache_dir: Option<PathBuf>,
    pub draft_k: usize,
    pub extra_models: Vec<ServerModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerModelConfig {
    pub name: String,
    pub model_dir: PathBuf,
    pub engine: String,
    pub q8_cache_dir: Option<PathBuf>,
    pub draft_model_dir: Option<PathBuf>,
    pub draft_engine: String,
    pub draft_cache_dir: Option<PathBuf>,
    pub draft_k: usize,
}

#[derive(Clone)]
struct ServerStateHandle(Arc<ServerStateInner>);

#[derive(Clone)]
struct ServerStateInner {
    default_model_name: String,
    models: Arc<HashMap<String, ModelWorkerHandle>>,
    next_request_id: Arc<AtomicU64>,
}

#[derive(Clone)]
struct ModelWorkerHandle(Arc<ModelWorker>);

struct ModelWorker {
    model_name: String,
    engine: String,
    model_dir: PathBuf,
    model: Arc<RuntimeModel>,
    draft_model: Option<Arc<RuntimeModel>>,
    tokenizer: Arc<Tokenizer>,
    scheduler: Arc<Mutex<RuntimeScheduler>>,
    max_new_tokens: usize,
    prefill_chunk_tokens: usize,
    kv_swap_dir: Option<PathBuf>,
    kv_max_resident_pages: usize,
    kv_swap_threshold: f32,
    draft_k: usize,
    request_tx: tokio::sync::mpsc::Sender<QueueItem>,
    num_requests: Arc<AtomicU64>,
    active_requests: Arc<AtomicU64>,
    prompt_tokens: Arc<AtomicU64>,
    completion_tokens: Arc<AtomicU64>,
    ngram_proposal_steps: Arc<AtomicU64>,
    ngram_proposed_tokens: Arc<AtomicU64>,
    ngram_accepted_tokens: Arc<AtomicU64>,
}

enum RuntimeModel {
    F32(NativeGemma),
    Q8(QuantizedGemma),
    Qwen35(Qwen35TextModel),
}

impl RuntimeModel {
    pub fn is_qwen35(&self) -> bool {
        matches!(self, Self::Qwen35(_))
    }

    pub fn as_qwen35(&self) -> Option<&Qwen35TextModel> {
        match self {
            Self::Qwen35(model) => Some(model),
            _ => None,
        }
    }

    pub fn new_cache_with_capacity(&self, max_seq: usize) -> AnyKvCache {
        match self {
            Self::F32(m) => m.new_cache_with_capacity(max_seq),
            Self::Q8(m) => m.new_cache_with_capacity(max_seq),
            Self::Qwen35(_) => panic!("Qwen3.5 uses Qwen35Cache, not AnyKvCache"),
        }
    }

    pub fn forward_token(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
    ) -> Vec<f32> {
        match self {
            Self::F32(m) => m.forward_token(token_id, position, cache),
            Self::Q8(m) => m.forward_token(token_id, position, cache),
            Self::Qwen35(_) => panic!("Qwen3.5 uses the serial Qwen worker path"),
        }
    }

    pub fn forward_token_output(
        &self,
        token_id: usize,
        position: usize,
        cache: &mut AnyKvCache,
    ) -> crate::model::ForwardOutput {
        match self {
            Self::F32(m) => {
                m.forward_token_with_lora_output(token_id, position, cache, m.lora.as_ref())
            }
            Self::Q8(m) => {
                m.forward_token_with_lora_output(token_id, position, cache, m.lora.as_ref())
            }
            Self::Qwen35(_) => panic!("Qwen3.5 uses the serial Qwen worker path"),
        }
    }

    pub fn forward_cuneiform_concepts_output(
        &self,
        concepts: &[crate::cuneiform::Concept6D],
        position: usize,
        cache: &mut AnyKvCache,
    ) -> crate::model::ForwardOutput {
        match self {
            Self::F32(m) => m.forward_cuneiform_concepts_output(concepts, position, cache),
            Self::Q8(m) => m.forward_cuneiform_concepts_output(concepts, position, cache),
            Self::Qwen35(_) => panic!("Qwen3.5 does not implement Cuneiform-U concept input"),
        }
    }

    pub fn hidden_size(&self) -> usize {
        match self {
            Self::F32(m) => m.cfg.hidden_size,
            Self::Q8(m) => m.cfg.hidden_size,
            Self::Qwen35(m) => m.cfg.hidden_size,
        }
    }

    pub fn vocab_size(&self) -> usize {
        match self {
            Self::F32(m) => m.cfg.vocab_size,
            Self::Q8(m) => m.cfg.vocab_size,
            Self::Qwen35(m) => m.cfg.vocab_size,
        }
    }

    pub fn max_position_embeddings(&self) -> usize {
        match self {
            Self::F32(m) => m.cfg.max_position_embeddings,
            Self::Q8(m) => m.cfg.max_position_embeddings,
            Self::Qwen35(m) => m.cfg.max_position_embeddings,
        }
    }

    pub fn forward_batch(
        &self,
        batch: &[(usize, usize)],
        caches: &mut [AnyKvCache],
    ) -> Vec<Vec<f32>> {
        match self {
            Self::F32(m) => m.forward_batch(batch, caches),
            Self::Q8(m) => m.forward_batch(batch, caches),
            Self::Qwen35(_) => panic!("Qwen3.5 uses the serial Qwen worker path"),
        }
    }

    pub fn forward_candidate_block(
        &self,
        sequence_id: u64,
        start_position: usize,
        tokens: &[usize],
        kv_cache: &mut crate::paged_kv::PagedKvCache,
    ) -> Vec<Vec<f32>> {
        if tokens.is_empty() {
            return Vec::new();
        }
        let cache_ptr = crate::model::SharedPagedKvCache(kv_cache as *mut _);
        let batch: Vec<_> = tokens
            .iter()
            .enumerate()
            .map(|(idx, &token)| (token, start_position + idx))
            .collect();
        let mut caches: Vec<_> = tokens
            .iter()
            .map(|_| AnyKvCache::Paged {
                cache: cache_ptr,
                sequence_id,
            })
            .collect();
        self.forward_batch(&batch, &mut caches)
    }

    fn layer_shapes(&self) -> Vec<(usize, usize)> {
        match self {
            Self::F32(m) => m
                .layers
                .iter()
                .map(|layer| {
                    let head_dim = layer.head_dim(&m.cfg);
                    (layer.kv_heads(&m.cfg), head_dim)
                })
                .collect(),
            Self::Q8(m) => m
                .layers
                .iter()
                .map(|layer| {
                    let head_dim = layer.head_dim(&m.cfg);
                    (layer.kv_heads(&m.cfg), head_dim)
                })
                .collect(),
            Self::Qwen35(_) => panic!("Qwen3.5 uses mixed KV caches, not Gemma paged KV shapes"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CompletionRequest {
    model: Option<String>,
    prompt: Option<PromptInput>,
    prompt_token_ids: Option<Vec<usize>>,
    max_tokens: Option<usize>,
    max_completion_tokens: Option<usize>,
    temperature: Option<f32>,
    top_k: Option<usize>,
    top_p: Option<f32>,
    min_p: Option<f32>,
    seed: Option<u64>,
    stop: Option<StopInput>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PromptInput {
    One(String),
    Many(Vec<String>),
    Tokens(Vec<usize>),
    ManyTokens(Vec<Vec<usize>>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum StopInput {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct ChatCompletionRequest {
    model: Option<String>,
    messages: Vec<ChatMessage>,
    max_tokens: Option<usize>,
    max_completion_tokens: Option<usize>,
    temperature: Option<f32>,
    top_k: Option<usize>,
    top_p: Option<f32>,
    min_p: Option<f32>,
    seed: Option<u64>,
    stop: Option<StopInput>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct HiddenConceptRequest {
    model: Option<String>,
    concepts: Vec<[u8; 6]>,
    max_tokens: Option<usize>,
    return_hidden: Option<bool>,
}

#[derive(Debug, Serialize)]
struct HiddenConceptResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    concept_count: usize,
    hidden_size: usize,
    vocab_size: usize,
    generated_token_ids: Vec<usize>,
    hidden_states: Option<Vec<Vec<f32>>>,
    top_logits: Vec<HiddenTopLogit>,
}

#[derive(Debug, Serialize)]
struct HiddenTopLogit {
    token_id: usize,
    logit: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: ChatContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ChatContent {
    Text(String),
    Parts(Vec<ChatContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatContentPart {
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ChatResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct CompletionResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<CompletionChoice>,
    usage: Usage,
    zymatica: Vec<ZymaticaPlan>,
}

#[derive(Debug, Serialize)]
struct CompletionChoice {
    text: String,
    index: usize,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<ChatChoice>,
    usage: Usage,
    zymatica: Vec<ZymaticaPlan>,
}

#[derive(Debug, Serialize)]
struct ChatChoice {
    index: usize,
    message: ChatResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Default, Serialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

type ApiError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, Serialize, Deserialize)]
struct ZymaticaPlan {
    request_id: u64,
    prompt_tokens: usize,
    reusable_prefix_tokens: usize,
    billable_tokens: usize,
    scheduler_total_billable_tokens: usize,
}

// OpenAI SSE Streaming structs
#[derive(Debug, Serialize)]
struct CompletionStreamChoice {
    text: String,
    index: usize,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompletionStreamResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<CompletionStreamChoice>,
}

#[derive(Debug, Serialize)]
struct ChatStreamDelta {
    content: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatStreamChoice {
    index: usize,
    delta: ChatStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatStreamResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Serialize)]
struct ModelsResponse {
    object: &'static str,
    data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
struct ModelInfo {
    id: String,
    object: &'static str,
    created: u64,
    owned_by: &'static str,
    zymatica: ModelInfoExtra,
}

#[derive(Debug, Serialize)]
struct ModelInfoExtra {
    engine: String,
    model_dir: String,
    draft_enabled: bool,
    draft_k: usize,
}

struct TokenStream {
    rx: tokio::sync::mpsc::Receiver<Result<String, String>>,
}

impl futures_util::stream::Stream for TokenStream {
    type Item = Result<Event, std::convert::Infallible>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(Ok(text))) => {
                std::task::Poll::Ready(Some(Ok(Event::default().data(text))))
            }
            std::task::Poll::Ready(Some(Err(err))) => {
                std::task::Poll::Ready(Some(Ok(Event::default().data(format!("Error: {err}")))))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

pub async fn serve(config: ServerConfig) -> Result<()> {
    warn_if_auth_is_not_configured(&config.bind);
    let tokenizer = Tokenizer::from_file(&config.tokenizer)
        .map_err(|e| anyhow::anyhow!("loading tokenizer {}: {e}", config.tokenizer.display()))?;
    let tokenizer = Arc::new(tokenizer);
    let selected_engine = resolve_server_engine(&config.model_dir, &config.engine);
    let default_model_name = format!("zymatica-{selected_engine}");
    let default_worker = build_model_worker(
        default_model_name.clone(),
        config.model_dir.clone(),
        selected_engine.clone(),
        config.q8_cache_dir.clone(),
        config.draft_model_dir.clone(),
        config.draft_engine.clone(),
        config.draft_cache_dir.clone(),
        config.draft_k,
        tokenizer.clone(),
        &config,
    )?;

    let mut models = HashMap::new();
    models.insert(default_model_name.clone(), default_worker);
    for model_config in &config.extra_models {
        if models.contains_key(&model_config.name) {
            bail!("duplicate model registry entry '{}'", model_config.name);
        }
        let selected_engine = resolve_server_engine(&model_config.model_dir, &model_config.engine);
        let worker = build_model_worker(
            model_config.name.clone(),
            model_config.model_dir.clone(),
            selected_engine,
            model_config.q8_cache_dir.clone(),
            model_config.draft_model_dir.clone(),
            model_config.draft_engine.clone(),
            model_config.draft_cache_dir.clone(),
            model_config.draft_k,
            tokenizer.clone(),
            &config,
        )?;
        models.insert(model_config.name.clone(), worker);
    }

    let state = ServerStateHandle(Arc::new(ServerStateInner {
        default_model_name,
        models: Arc::new(models),
        next_request_id: Arc::new(AtomicU64::new(1)),
    }));

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/mcp/manifest", get(mcp_manifest))
        .route("/.well-known/agent-card.json", get(agent_card))
        .route("/metrics", get(metrics))
        .route("/v1/models", get(list_models))
        .route("/v1/completions", post(completions))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/hidden/concepts", post(hidden_concepts))
        .route("/mcp", post(handle_mcp))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(&config.bind)
        .await
        .with_context(|| format!("binding {}", config.bind))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .with_context(|| format!("serving {}", config.bind))?;
    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("warning: failed to install Ctrl-C shutdown handler: {error}");
    }
}

fn warn_if_auth_is_not_configured(bind: &str) {
    let has_key = std::env::var("ZYMATICA_API_KEY")
        .map(|key| !key.trim().is_empty())
        .unwrap_or(false);
    if has_key {
        return;
    }
    eprintln!(
        "warning: ZYMATICA_API_KEY is not set; inference endpoints will accept unauthenticated requests on {bind}"
    );
}

fn load_runtime_model(
    model_dir: &Path,
    engine: &str,
    cache_dir: Option<&Path>,
) -> Result<RuntimeModel> {
    match engine {
        "f32" => {
            if qwen35::is_qwen35_dir(model_dir) {
                Ok(RuntimeModel::Qwen35(
                    Qwen35TextModel::from_hf_dir(model_dir).with_context(|| {
                        format!("loading Qwen3.5 f32 model {}", model_dir.display())
                    })?,
                ))
            } else {
                Ok(RuntimeModel::F32(
                    NativeGemma::from_hf_dir(model_dir)
                        .with_context(|| format!("loading f32 model {}", model_dir.display()))?,
                ))
            }
        }
        "q8" | "q8i" | "q5" | "q4" | "q3" | "q3-gpu" => {
            let mode = match engine {
                "q8" | "q8i" => QuantMode::Q8,
                "q5" => QuantMode::Q5,
                "q4" => QuantMode::Q4,
                "q3" | "q3-gpu" => QuantMode::Q3,
                _ => unreachable!(),
            };
            if qwen35::is_qwen35_dir(model_dir) {
                if engine == "q3-gpu" {
                    bail!("engine 'q3-gpu' currently supports Gemma models, not Qwen3.5");
                }
                let model = if let Some(cache_dir) = cache_dir {
                    Qwen35TextModel::from_hf_dir_with_cache_and_mode(model_dir, cache_dir, mode)
                        .with_context(|| {
                            format!(
                                "loading Qwen3.5 {:?} model {} with cache {}",
                                mode,
                                model_dir.display(),
                                cache_dir.display()
                            )
                        })?
                } else {
                    Qwen35TextModel::from_hf_dir_with_mode(model_dir, mode).with_context(|| {
                        format!("loading Qwen3.5 {:?} model {}", mode, model_dir.display())
                    })?
                };
                return Ok(RuntimeModel::Qwen35(model));
            }
            let activation_mode = match engine {
                "q8i" => QuantizedActivationMode::DynamicInt8,
                "q3-gpu" => QuantizedActivationMode::GpuF32,
                _ => QuantizedActivationMode::F32,
            };
            let model = if let Some(cache_dir) = cache_dir {
                QuantizedGemma::from_hf_dir_with_cache_and_mode(model_dir, cache_dir, mode)
                    .with_context(|| {
                        format!(
                            "loading quantized {:?} model {} with cache {}",
                            mode,
                            model_dir.display(),
                            cache_dir.display()
                        )
                    })?
            } else {
                QuantizedGemma::from_hf_dir_with_mode(model_dir, mode).with_context(|| {
                    format!("loading quantized {:?} model {}", mode, model_dir.display())
                })?
            }
            .with_activation_mode(activation_mode);
            let model = if activation_mode == QuantizedActivationMode::GpuF32 {
                model.with_q3_gpu()?
            } else {
                model
            };
            Ok(RuntimeModel::Q8(model))
        }
        other => {
            bail!(
                "unsupported engine '{other}', expected 'f32', 'q8', 'q8i', 'q5', 'q4', 'q3', or 'q3-gpu'"
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn build_model_worker(
    model_name: String,
    model_dir: PathBuf,
    engine: String,
    q8_cache_dir: Option<PathBuf>,
    draft_model_dir: Option<PathBuf>,
    draft_engine: String,
    draft_cache_dir: Option<PathBuf>,
    draft_k: usize,
    tokenizer: Arc<Tokenizer>,
    config: &ServerConfig,
) -> Result<ModelWorkerHandle> {
    let model = load_runtime_model(&model_dir, &engine, q8_cache_dir.as_deref())?;
    if model.is_qwen35() && draft_model_dir.is_some() {
        bail!(
            "Qwen3.5 server support currently uses the serial native worker and does not support draft models"
        );
    }
    let draft_model = if let Some(draft_model_dir) = &draft_model_dir {
        Some(load_runtime_model(
            draft_model_dir,
            &draft_engine,
            draft_cache_dir.as_deref(),
        )?)
    } else {
        None
    };
    let (request_tx, request_rx) = tokio::sync::mpsc::channel::<QueueItem>(REQUEST_QUEUE_CAPACITY);
    let worker = ModelWorkerHandle(Arc::new(ModelWorker {
        model_name,
        engine,
        model_dir,
        model: Arc::new(model),
        draft_model: draft_model.map(Arc::new),
        tokenizer,
        scheduler: Arc::new(Mutex::new(RuntimeScheduler::new(
            config.scheduler_max_batch_tokens,
        ))),
        max_new_tokens: config.max_new_tokens,
        prefill_chunk_tokens: if config.prefill_chunk_tokens == 0 {
            DEFAULT_PREFILL_CHUNK_TOKENS
        } else {
            config.prefill_chunk_tokens
        },
        kv_swap_dir: config.kv_swap_dir.clone(),
        kv_max_resident_pages: config.kv_max_resident_pages,
        kv_swap_threshold: config.kv_swap_threshold,
        draft_k,
        request_tx,
        num_requests: Arc::new(AtomicU64::new(0)),
        active_requests: Arc::new(AtomicU64::new(0)),
        prompt_tokens: Arc::new(AtomicU64::new(0)),
        completion_tokens: Arc::new(AtomicU64::new(0)),
        ngram_proposal_steps: Arc::new(AtomicU64::new(0)),
        ngram_proposed_tokens: Arc::new(AtomicU64::new(0)),
        ngram_accepted_tokens: Arc::new(AtomicU64::new(0)),
    }));

    let worker_for_thread = worker.clone();
    std::thread::spawn(move || {
        let name = worker_for_thread.0.model_name.clone();
        let result = if worker_for_thread.0.model.is_qwen35() {
            run_serial_qwen35_worker(worker_for_thread, request_rx)
        } else {
            run_continuous_batcher(worker_for_thread, request_rx)
        };
        if let Err(e) = result {
            eprintln!("Model worker for {name} failed: {e}");
        }
    });
    Ok(worker)
}

fn resolve_server_engine(_model_dir: &Path, engine: &str) -> String {
    if engine == "auto" {
        let decision = decide_quant_mode(&EdgeDeviceProfile::detect(), AutoPriority::from_env());
        eprintln!(
            "zymatica auto engine selected {} for registry model: {}",
            decision.engine_name(),
            decision.reason
        );
        decision.engine_name().to_string()
    } else {
        engine.to_string()
    }
}

async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "runtime": "zymatica-engine"
    }))
}

async fn mcp_manifest(headers: axum::http::HeaderMap) -> Result<impl IntoResponse, ApiError> {
    check_auth(&headers)?;
    Ok(Json(agent_runtime::mcp_manifest()))
}

async fn agent_card(headers: axum::http::HeaderMap) -> Result<impl IntoResponse, ApiError> {
    check_auth(&headers)?;
    let keypair = agent_runtime::AgentKeypair::from_seed([9_u8; 32]);
    Ok(Json(agent_runtime::agent_card(keypair.identity())))
}

async fn list_models(
    State(state): State<ServerStateHandle>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    check_auth(&headers)?;
    let mut data: Vec<_> = state
        .0
        .models
        .values()
        .map(|worker| ModelInfo {
            id: worker.0.model_name.clone(),
            object: "model",
            created: 0,
            owned_by: "zymatica",
            zymatica: ModelInfoExtra {
                engine: worker.0.engine.clone(),
                model_dir: worker.0.model_dir.to_string_lossy().into_owned(),
                draft_enabled: worker.0.draft_k > 0 && !worker.0.model.is_qwen35(),
                draft_k: worker.0.draft_k,
            },
        })
        .collect();
    data.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Json(ModelsResponse {
        object: "list",
        data,
    }))
}

async fn metrics(
    State(state): State<ServerStateHandle>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    check_auth(&headers)?;

    let mut body = String::from(
        "# HELP zymatica_requests_total Total number of inference requests processed\n\
         # TYPE zymatica_requests_total counter\n\
         # HELP zymatica_active_requests Number of currently active inference requests\n\
         # TYPE zymatica_active_requests gauge\n\
         # HELP zymatica_prompt_tokens_total Total prompt tokens processed\n\
         # TYPE zymatica_prompt_tokens_total counter\n\
         # HELP zymatica_completion_tokens_total Total completion tokens generated\n\
         # TYPE zymatica_completion_tokens_total counter\n\
         # HELP zymatica_ngram_proposal_steps_total N-gram speculative verification steps\n\
         # TYPE zymatica_ngram_proposal_steps_total counter\n\
         # HELP zymatica_ngram_proposed_tokens_total Tokens proposed by the online n-gram engine\n\
         # TYPE zymatica_ngram_proposed_tokens_total counter\n\
         # HELP zymatica_ngram_accepted_tokens_total N-gram tokens accepted by target verification\n\
         # TYPE zymatica_ngram_accepted_tokens_total counter\n",
    );
    let mut workers: Vec<_> = state.0.models.values().collect();
    workers.sort_by(|a, b| a.0.model_name.cmp(&b.0.model_name));
    for worker in workers {
        body.push_str(&format!(
            "zymatica_requests_total{{model=\"{}\"}} {}\n\
             zymatica_active_requests{{model=\"{}\"}} {}\n\
             zymatica_prompt_tokens_total{{model=\"{}\"}} {}\n\
             zymatica_completion_tokens_total{{model=\"{}\"}} {}\n\
             zymatica_ngram_proposal_steps_total{{model=\"{}\"}} {}\n\
             zymatica_ngram_proposed_tokens_total{{model=\"{}\"}} {}\n\
             zymatica_ngram_accepted_tokens_total{{model=\"{}\"}} {}\n",
            worker.0.model_name,
            worker.0.num_requests.load(Ordering::Relaxed),
            worker.0.model_name,
            worker.0.active_requests.load(Ordering::Relaxed),
            worker.0.model_name,
            worker.0.prompt_tokens.load(Ordering::Relaxed),
            worker.0.model_name,
            worker.0.completion_tokens.load(Ordering::Relaxed),
            worker.0.model_name,
            worker.0.ngram_proposal_steps.load(Ordering::Relaxed),
            worker.0.model_name,
            worker.0.ngram_proposed_tokens.load(Ordering::Relaxed),
            worker.0.model_name,
            worker.0.ngram_accepted_tokens.load(Ordering::Relaxed)
        ));
    }

    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        body,
    ))
}

async fn completions(
    State(state): State<ServerStateHandle>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CompletionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_auth(&headers)?;

    let worker = resolve_model_worker(&state, request.model.as_deref())?;
    let model_name = worker.0.model_name.clone();
    let prompt_ids =
        resolve_completion_prompt_ids(&worker, request.prompt, request.prompt_token_ids)?;
    let prompt_len = prompt_ids.len();
    let max_tokens = resolve_max_tokens(request.max_tokens, request.max_completion_tokens)?;
    validate_context_window(
        prompt_len,
        max_tokens,
        worker.0.model.max_position_embeddings(),
    )?;
    let stop_sequences = normalize_stop_sequences(request.stop)?;
    let temperature = request.temperature.unwrap_or(0.0);
    let top_k = request.top_k.unwrap_or(1);
    let seed = request.seed.unwrap_or(0);

    if request.stream.unwrap_or(false) {
        let mut worker_rx = queue_request(
            &state,
            &worker,
            QueueRequest {
                prompt_ids: prompt_ids.clone(),
                max_tokens,
                temperature,
                top_k,
                top_p: request.top_p,
                min_p: request.min_p,
                seed,
                stop_sequences: stop_sequences.clone(),
            },
        )?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let id = format!("cmpl-{}", now_unix_seconds());
        let model_name_clone = model_name.clone();

        tokio::spawn(async move {
            let created = now_unix_seconds();
            while let Some(msg) = worker_rx.recv().await {
                match msg {
                    Ok(serialized) => {
                        if let Ok(worker_msg) = serde_json::from_str::<WorkerMessage>(&serialized) {
                            match worker_msg {
                                WorkerMessage::Delta { text } => {
                                    let resp = CompletionStreamResponse {
                                        id: id.clone(),
                                        object: "text_completion",
                                        created,
                                        model: model_name_clone.clone(),
                                        choices: vec![CompletionStreamChoice {
                                            text,
                                            index: 0,
                                            finish_reason: None,
                                        }],
                                    };
                                    if tx.send(serialize_channel_message(&resp)).await.is_err() {
                                        break;
                                    }
                                }
                                WorkerMessage::Done { finish_reason, .. } => {
                                    let resp = CompletionStreamResponse {
                                        id: id.clone(),
                                        object: "text_completion",
                                        created,
                                        model: model_name_clone.clone(),
                                        choices: vec![CompletionStreamChoice {
                                            text: "".to_string(),
                                            index: 0,
                                            finish_reason: Some(finish_reason),
                                        }],
                                    };
                                    let _ = tx.send(serialize_channel_message(&resp)).await;
                                    let _ = tx.send(Ok("[DONE]".to_string())).await;
                                    break;
                                }
                                WorkerMessage::Error { message } => {
                                    let _ = tx.send(Err(message)).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        return Ok(Sse::new(TokenStream { rx })
            .keep_alive(
                axum::response::sse::KeepAlive::new()
                    .interval(Duration::from_secs(1))
                    .text("keep-alive"),
            )
            .into_response());
    }

    let mut worker_rx = queue_request(
        &state,
        &worker,
        QueueRequest {
            prompt_ids,
            max_tokens,
            temperature,
            top_k,
            top_p: request.top_p,
            min_p: request.min_p,
            seed,
            stop_sequences,
        },
    )?;
    let mut text = String::new();
    let mut prompt_tokens = 0;
    let mut completion_tokens = 0;
    let mut finish_reason = "length".to_string();
    let mut plan = ZymaticaPlan {
        request_id: 0,
        prompt_tokens: 0,
        reusable_prefix_tokens: 0,
        billable_tokens: 0,
        scheduler_total_billable_tokens: 0,
    };

    while let Some(msg) = worker_rx.recv().await {
        match msg {
            Ok(serialized) => {
                if let Ok(worker_msg) = serde_json::from_str::<WorkerMessage>(&serialized) {
                    match worker_msg {
                        WorkerMessage::Delta { text: d } => {
                            text.push_str(&d);
                        }
                        WorkerMessage::Done {
                            text: final_text,
                            prompt_tokens: pt,
                            completion_tokens: ct,
                            finish_reason: reason,
                            plan: p,
                        } => {
                            text = final_text;
                            prompt_tokens = pt;
                            completion_tokens = ct;
                            finish_reason = reason;
                            plan = p;
                        }
                        WorkerMessage::Error { message } => {
                            return Err(internal_error(message));
                        }
                    }
                }
            }
            Err(e) => return Err(internal_error(e)),
        }
    }

    let usage = Usage {
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
    };
    let choices = vec![CompletionChoice {
        text,
        index: 0,
        finish_reason,
    }];

    Ok(Json(CompletionResponse {
        id: format!("cmpl-{}", now_unix_seconds()),
        object: "text_completion",
        created: now_unix_seconds(),
        model: model_name,
        choices,
        usage,
        zymatica: vec![plan],
    })
    .into_response())
}

async fn chat_completions(
    State(state): State<ServerStateHandle>,
    headers: axum::http::HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_auth(&headers)?;

    if request.messages.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "messages must not be empty",
        ));
    }

    let prompt = render_chat_prompt(&request.messages)?;
    if prompt.trim().is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Prompt must not be empty",
        ));
    }

    let worker = resolve_model_worker(&state, request.model.as_deref())?;
    let model_name = worker.0.model_name.clone();
    let prompt_ids = encode_prompt_ids_for_worker(&worker, &prompt)?;
    let prompt_len = prompt_ids.len();
    let max_tokens = resolve_max_tokens(request.max_tokens, request.max_completion_tokens)?;
    validate_context_window(
        prompt_len,
        max_tokens,
        worker.0.model.max_position_embeddings(),
    )?;
    let stop_sequences = normalize_stop_sequences(request.stop)?;
    let temperature = request.temperature.unwrap_or(0.0);
    let top_k = request.top_k.unwrap_or(1);
    let seed = request.seed.unwrap_or(0);

    if request.stream.unwrap_or(false) {
        let mut worker_rx = queue_request(
            &state,
            &worker,
            QueueRequest {
                prompt_ids: prompt_ids.clone(),
                max_tokens,
                temperature,
                top_k,
                top_p: request.top_p,
                min_p: request.min_p,
                seed,
                stop_sequences: stop_sequences.clone(),
            },
        )?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let id = format!("chatcmpl-{}", now_unix_seconds());
        let model_name_clone = model_name.clone();

        tokio::spawn(async move {
            let created = now_unix_seconds();
            while let Some(msg) = worker_rx.recv().await {
                match msg {
                    Ok(serialized) => {
                        if let Ok(worker_msg) = serde_json::from_str::<WorkerMessage>(&serialized) {
                            match worker_msg {
                                WorkerMessage::Delta { text } => {
                                    let resp = ChatStreamResponse {
                                        id: id.clone(),
                                        object: "chat.completion.chunk",
                                        created,
                                        model: model_name_clone.clone(),
                                        choices: vec![ChatStreamChoice {
                                            index: 0,
                                            delta: ChatStreamDelta {
                                                content: Some(text),
                                            },
                                            finish_reason: None,
                                        }],
                                    };
                                    if tx.send(serialize_channel_message(&resp)).await.is_err() {
                                        break;
                                    }
                                }
                                WorkerMessage::Done { finish_reason, .. } => {
                                    let resp = ChatStreamResponse {
                                        id: id.clone(),
                                        object: "chat.completion.chunk",
                                        created,
                                        model: model_name_clone.clone(),
                                        choices: vec![ChatStreamChoice {
                                            index: 0,
                                            delta: ChatStreamDelta { content: None },
                                            finish_reason: Some(finish_reason),
                                        }],
                                    };
                                    let _ = tx.send(serialize_channel_message(&resp)).await;
                                    let _ = tx.send(Ok("[DONE]".to_string())).await;
                                    break;
                                }
                                WorkerMessage::Error { message } => {
                                    let _ = tx.send(Err(message)).await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        return Ok(Sse::new(TokenStream { rx })
            .keep_alive(
                axum::response::sse::KeepAlive::new()
                    .interval(Duration::from_secs(1))
                    .text("keep-alive"),
            )
            .into_response());
    }

    let mut worker_rx = queue_request(
        &state,
        &worker,
        QueueRequest {
            prompt_ids,
            max_tokens,
            temperature,
            top_k,
            top_p: request.top_p,
            min_p: request.min_p,
            seed,
            stop_sequences,
        },
    )?;
    let mut text = String::new();
    let mut prompt_tokens = 0;
    let mut completion_tokens = 0;
    let mut finish_reason = "length".to_string();
    let mut plan = ZymaticaPlan {
        request_id: 0,
        prompt_tokens: 0,
        reusable_prefix_tokens: 0,
        billable_tokens: 0,
        scheduler_total_billable_tokens: 0,
    };

    while let Some(msg) = worker_rx.recv().await {
        match msg {
            Ok(serialized) => {
                if let Ok(worker_msg) = serde_json::from_str::<WorkerMessage>(&serialized) {
                    match worker_msg {
                        WorkerMessage::Delta { text: d } => {
                            text.push_str(&d);
                        }
                        WorkerMessage::Done {
                            text: final_text,
                            prompt_tokens: pt,
                            completion_tokens: ct,
                            finish_reason: reason,
                            plan: p,
                        } => {
                            text = final_text;
                            prompt_tokens = pt;
                            completion_tokens = ct;
                            finish_reason = reason;
                            plan = p;
                        }
                        WorkerMessage::Error { message } => {
                            return Err(internal_error(message));
                        }
                    }
                }
            }
            Err(e) => return Err(internal_error(e)),
        }
    }

    let usage = Usage {
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
    };

    Ok(Json(ChatCompletionResponse {
        id: format!("chatcmpl-{}", now_unix_seconds()),
        object: "chat.completion",
        created: now_unix_seconds(),
        model: model_name,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatResponseMessage {
                role: "assistant".to_string(),
                content: text,
            },
            finish_reason,
        }],
        usage,
        zymatica: vec![plan],
    })
    .into_response())
}

async fn hidden_concepts(
    State(state): State<ServerStateHandle>,
    headers: axum::http::HeaderMap,
    Json(request): Json<HiddenConceptRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_auth(&headers)?;

    if request.concepts.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "concepts must not be empty",
        ));
    }
    if request.concepts.len() > 256 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "concepts may contain at most 256 coordinates",
        ));
    }

    let mut concepts = Vec::with_capacity(request.concepts.len());
    for axes in request.concepts {
        if axes.iter().any(|axis| *axis >= 16) {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                "Cuneiform-U concept axes must be 4-bit values in 0..15",
            ));
        }
        concepts.push(crate::cuneiform::Concept6D::new(
            axes[0], axes[1], axes[2], axes[3], axes[4], axes[5],
        ));
    }

    let max_tokens = request.max_tokens.unwrap_or(1);
    if max_tokens == 0 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "max_tokens must be greater than zero",
        ));
    }
    if max_tokens > 128 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "hidden concept endpoint supports max_tokens <= 128",
        ));
    }

    let worker = resolve_model_worker(&state, request.model.as_deref())?;
    if worker.0.model.is_qwen35() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Qwen3.5 models do not support the Cuneiform-U hidden concept endpoint",
        ));
    }
    let model_name = worker.0.model_name.clone();
    let model = worker.0.model.clone();
    let return_hidden = request.return_hidden.unwrap_or(true);
    let concept_count = concepts.len();
    let hidden_size = model.hidden_size();
    let vocab_size = model.vocab_size();
    let active_requests = worker.0.active_requests.clone();
    let completion_tokens = worker.0.completion_tokens.clone();
    worker.0.num_requests.fetch_add(1, Ordering::Relaxed);
    active_requests.fetch_add(1, Ordering::Relaxed);

    let response = tokio::task::spawn_blocking(move || -> Result<HiddenConceptResponse> {
        let mut cache = model.new_cache_with_capacity(max_tokens + 1);
        let mut output = model.forward_cuneiform_concepts_output(&concepts, 0, &mut cache);
        let mut hidden_states = return_hidden.then(Vec::new);
        if let Some(states) = hidden_states.as_mut() {
            states.push(output.hidden_state.clone());
        }

        let mut generated_token_ids = Vec::with_capacity(max_tokens);
        let mut top_logits = select_top_logits(&output.logits, 8);
        for idx in 0..max_tokens {
            let next = crate::ops::argmax(&output.logits);
            generated_token_ids.push(next);
            if idx + 1 < max_tokens {
                output = model.forward_token_output(next, idx + 1, &mut cache);
                top_logits = select_top_logits(&output.logits, 8);
                if let Some(states) = hidden_states.as_mut() {
                    states.push(output.hidden_state.clone());
                }
            }
        }

        Ok(HiddenConceptResponse {
            id: format!("hidden-{}", now_unix_seconds()),
            object: "zymatica.hidden_concept_completion",
            created: now_unix_seconds(),
            model: model_name,
            concept_count,
            hidden_size,
            vocab_size,
            generated_token_ids,
            hidden_states,
            top_logits,
        })
    })
    .await;
    active_requests.fetch_sub(1, Ordering::Relaxed);
    let response =
        response.map_err(|e| internal_error(format!("hidden concept worker failed: {e}")))?;
    match response {
        Ok(response) => {
            completion_tokens
                .fetch_add(response.generated_token_ids.len() as u64, Ordering::Relaxed);
            Ok(Json(response).into_response())
        }
        Err(err) => Err(internal_error(err.to_string())),
    }
}

fn select_top_logits(logits: &[f32], limit: usize) -> Vec<HiddenTopLogit> {
    let mut indexed: Vec<_> = logits
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| value.is_finite())
        .collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    indexed
        .into_iter()
        .take(limit)
        .map(|(token_id, logit)| HiddenTopLogit { token_id, logit })
        .collect()
}

fn resolve_completion_prompt_ids(
    worker: &ModelWorkerHandle,
    prompt: Option<PromptInput>,
    prompt_token_ids: Option<Vec<usize>>,
) -> Result<Vec<usize>, ApiError> {
    if prompt.is_some() && prompt_token_ids.is_some() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Use either prompt or prompt_token_ids, not both",
        ));
    }
    if let Some(prompt_token_ids) = prompt_token_ids {
        validate_prompt_ids_for_model(&prompt_token_ids, worker.0.model.vocab_size())
            .map_err(|message| api_error(StatusCode::BAD_REQUEST, message))?;
        return Ok(prompt_token_ids);
    }

    match prompt {
        Some(PromptInput::One(prompt)) => {
            if prompt.trim().is_empty() {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    "Prompt must not be empty",
                ));
            }
            encode_prompt_ids_for_worker(worker, &prompt)
        }
        Some(PromptInput::Many(prompts)) => {
            let prompt = only_single_prompt(prompts.len()).and_then(|_| {
                prompts
                    .first()
                    .cloned()
                    .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "Prompt must not be empty"))
            })?;
            if prompt.trim().is_empty() {
                return Err(api_error(
                    StatusCode::BAD_REQUEST,
                    "Prompt must not be empty",
                ));
            }
            encode_prompt_ids_for_worker(worker, &prompt)
        }
        Some(PromptInput::Tokens(prompt_ids)) => {
            validate_prompt_ids_for_model(&prompt_ids, worker.0.model.vocab_size())
                .map_err(|message| api_error(StatusCode::BAD_REQUEST, message))?;
            Ok(prompt_ids)
        }
        Some(PromptInput::ManyTokens(prompts)) => {
            only_single_prompt(prompts.len())?;
            let prompt_ids = prompts
                .first()
                .cloned()
                .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "Prompt must not be empty"))?;
            validate_prompt_ids_for_model(&prompt_ids, worker.0.model.vocab_size())
                .map_err(|message| api_error(StatusCode::BAD_REQUEST, message))?;
            Ok(prompt_ids)
        }
        None => Err(api_error(
            StatusCode::BAD_REQUEST,
            "prompt or prompt_token_ids is required",
        )),
    }
}

fn only_single_prompt(count: usize) -> Result<(), ApiError> {
    if count == 0 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Prompt must not be empty",
        ));
    }
    if count > 1 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Multiple prompts are not supported by this server",
        ));
    }
    Ok(())
}

fn render_chat_prompt(messages: &[ChatMessage]) -> Result<String, ApiError> {
    messages
        .iter()
        .map(|message| {
            let content = message
                .content
                .to_text()
                .map_err(|message| api_error(StatusCode::BAD_REQUEST, message))?;
            Ok(format!("{}: {}", message.role, content))
        })
        .collect::<std::result::Result<Vec<_>, ApiError>>()
        .map(|lines| lines.join("\n"))
}

impl ChatContent {
    fn to_text(&self) -> std::result::Result<String, String> {
        match self {
            ChatContent::Text(text) => Ok(text.clone()),
            ChatContent::Parts(parts) => {
                let mut text = String::new();
                for part in parts {
                    let kind = part.kind.as_deref().unwrap_or("text");
                    match kind {
                        "text" | "input_text" => {
                            let Some(part_text) = part.text.as_deref() else {
                                return Err(format!(
                                    "chat content part type '{kind}' is missing text"
                                ));
                            };
                            text.push_str(part_text);
                        }
                        other => {
                            return Err(format!(
                                "unsupported chat content part type '{other}'; this server accepts text only"
                            ));
                        }
                    }
                }
                Ok(text)
            }
        }
    }
}

fn resolve_max_tokens(
    max_tokens: Option<usize>,
    max_completion_tokens: Option<usize>,
) -> Result<usize, ApiError> {
    let max_tokens = match (max_tokens, max_completion_tokens) {
        (Some(left), Some(right)) if left != right => {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                "max_tokens and max_completion_tokens disagree",
            ));
        }
        (Some(value), _) | (_, Some(value)) => value,
        (None, None) => 16,
    };
    if max_tokens == 0 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "max_tokens must be greater than zero",
        ));
    }
    Ok(max_tokens)
}

fn validate_context_window(
    prompt_len: usize,
    max_tokens: usize,
    max_context_tokens: usize,
) -> Result<(), ApiError> {
    if prompt_len > 3072 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            format!("Prompt too long: {} tokens (max 3072)", prompt_len),
        ));
    }
    if max_tokens > 2048 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            format!("Requested max_tokens too high: {} (max 2048)", max_tokens),
        ));
    }
    if prompt_len + max_tokens > max_context_tokens {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            format!(
                "Total context length {} exceeds maximum of {} tokens",
                prompt_len + max_tokens,
                max_context_tokens
            ),
        ));
    }
    Ok(())
}

fn normalize_stop_sequences(stop: Option<StopInput>) -> Result<Vec<String>, ApiError> {
    let sequences = match stop {
        Some(StopInput::One(sequence)) => vec![sequence],
        Some(StopInput::Many(sequences)) => sequences,
        None => Vec::new(),
    };
    if sequences.len() > 4 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "stop may contain at most 4 sequences",
        ));
    }
    if sequences.iter().any(|sequence| sequence.is_empty()) {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "stop sequences must not be empty",
        ));
    }
    Ok(sequences)
}

fn queue_request(
    state: &ServerStateHandle,
    worker: &ModelWorkerHandle,
    request: QueueRequest,
) -> Result<tokio::sync::mpsc::Receiver<Result<String, String>>, ApiError> {
    validate_prompt_ids_for_model(&request.prompt_ids, worker.0.model.vocab_size())
        .map_err(|message| api_error(StatusCode::BAD_REQUEST, message))?;
    let request_id = state.0.next_request_id.fetch_add(1, Ordering::Relaxed);
    let (response_tx, response_rx) =
        tokio::sync::mpsc::channel::<Result<String, String>>(RESPONSE_QUEUE_CAPACITY);
    let item = QueueItem {
        request_id,
        prompt_ids: request.prompt_ids,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_k: request.top_k,
        top_p: request.top_p,
        min_p: request.min_p,
        seed: request.seed,
        stop_sequences: request.stop_sequences,
        response_tx,
    };
    match worker.0.request_tx.try_send(item) {
        Ok(()) => {
            worker.0.num_requests.fetch_add(1, Ordering::Relaxed);
            worker.0.active_requests.fetch_add(1, Ordering::Relaxed);
            Ok(response_rx)
        }
        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => Err(api_error(
            StatusCode::TOO_MANY_REQUESTS,
            "Server overloaded: request queue is full",
        )),
        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
            Err(internal_error("Server worker is not running"))
        }
    }
}

fn encode_prompt_ids_for_worker(
    worker: &ModelWorkerHandle,
    prompt: &str,
) -> Result<Vec<usize>, ApiError> {
    let encoded = worker
        .0
        .tokenizer
        .encode(prompt.to_string(), true)
        .map_err(|e| api_error(StatusCode::BAD_REQUEST, format!("Encoding prompt: {e}")))?;
    let prompt_ids: Vec<usize> = encoded.get_ids().iter().map(|id| *id as usize).collect();
    validate_prompt_ids_for_model(&prompt_ids, worker.0.model.vocab_size())
        .map_err(|message| api_error(StatusCode::BAD_REQUEST, message))?;
    Ok(prompt_ids)
}

fn validate_prompt_ids_for_model(
    prompt_ids: &[usize],
    vocab_size: usize,
) -> std::result::Result<(), String> {
    if prompt_ids.is_empty() {
        return Err("Prompt did not encode to any token ids".to_string());
    }
    if let Some(token_id) = prompt_ids
        .iter()
        .copied()
        .find(|&token_id| token_id >= vocab_size)
    {
        return Err(format!(
            "Prompt token id {token_id} is outside loaded model vocab size {vocab_size}"
        ));
    }
    Ok(())
}

fn init_active_request(item: QueueItem, worker: &ModelWorkerHandle) -> Result<ActiveRequest> {
    validate_prompt_ids_for_model(&item.prompt_ids, worker.0.model.vocab_size())
        .map_err(anyhow::Error::msg)?;

    let rng = rand::SeedableRng::seed_from_u64(item.seed);
    let request_is_greedy = item.temperature <= 0.0 && item.top_k <= 1;
    let draft_model_enabled =
        request_is_greedy && worker.0.draft_model.is_some() && worker.0.draft_k > 0;
    let ngram_enabled = request_is_greedy && worker.0.draft_model.is_none() && worker.0.draft_k > 0;
    let draft_cache = if draft_model_enabled {
        worker.0.draft_model.as_ref().map(|model| {
            model.new_cache_with_capacity(
                item.prompt_ids.len() + item.max_tokens + worker.0.draft_k + 1,
            )
        })
    } else {
        None
    };
    let ngram_proposer = ngram_enabled.then(|| {
        let mut proposer = FastNGramProposalEngine::new(3);
        proposer.train_tokens(&item.prompt_ids);
        proposer
    });

    Ok(ActiveRequest {
        id: item.request_id,
        prompt_ids: item.prompt_ids,
        generated_ids: Vec::new(),
        max_new_tokens: item.max_tokens,
        sampling: SamplingConfig {
            temperature: item.temperature,
            top_k: item.top_k,
            top_p: item.top_p,
            min_p: item.min_p,
        },
        rng,
        response_tx: item.response_tx,
        created_time: Instant::now(),
        last_logits: Vec::new(),
        draft_cache,
        draft_last_logits: Vec::new(),
        draft_prefill_pos: 0,
        draft_controller: (draft_model_enabled || ngram_enabled)
            .then(|| AdaptiveDraftController::new(1, worker.0.draft_k)),
        ngram_proposer,
        prefill_done: false,
        prefill_pos: 0,
        last_touched_iteration: 0,
        kv_swapped_path: None,
        kv_swap_count: 0,
        decoded_len: 0,
        stop_sequences: item.stop_sequences,
        reusable_prefix_tokens: 0,
        metrics: RequestMetrics::default(),
    })
}

fn run_serial_qwen35_worker(
    worker: ModelWorkerHandle,
    mut request_rx: tokio::sync::mpsc::Receiver<QueueItem>,
) -> Result<()> {
    while let Some(item) = request_rx.blocking_recv() {
        let response_tx = item.response_tx.clone();
        let prompt_tokens = item.prompt_ids.len();
        let result = process_serial_qwen35_request(&worker, item);
        match result {
            Ok(completion_tokens) => {
                worker
                    .0
                    .prompt_tokens
                    .fetch_add(prompt_tokens as u64, Ordering::Relaxed);
                worker
                    .0
                    .completion_tokens
                    .fetch_add(completion_tokens as u64, Ordering::Relaxed);
            }
            Err(err) => {
                let msg = WorkerMessage::Error {
                    message: err.to_string(),
                };
                let _ = response_tx.blocking_send(serialize_channel_message(&msg));
            }
        }
        worker.0.active_requests.fetch_sub(1, Ordering::Relaxed);
    }
    Ok(())
}

fn process_serial_qwen35_request(worker: &ModelWorkerHandle, item: QueueItem) -> Result<usize> {
    let model = worker
        .0
        .model
        .as_qwen35()
        .context("serial Qwen3.5 worker received a non-Qwen model")?;
    validate_prompt_ids_for_model(&item.prompt_ids, model.cfg.vocab_size)
        .map_err(anyhow::Error::msg)?;

    if item.response_tx.is_closed() {
        return Ok(0);
    }

    let max_tokens = item.max_tokens.clamp(1, worker.0.max_new_tokens.max(1));
    let sampling = SamplingConfig {
        temperature: item.temperature,
        top_k: item.top_k,
        top_p: item.top_p,
        min_p: item.min_p,
    };
    let mut rng = StdRng::seed_from_u64(item.seed);
    let mut cache = model.new_cache_with_capacity(item.prompt_ids.len() + max_tokens + 1);
    let mut logits = Vec::new();
    for (pos, token_id) in item.prompt_ids.iter().copied().enumerate() {
        logits = model.forward_token(token_id, pos, &mut cache);
    }
    if logits.is_empty() {
        bail!("Qwen3.5 request has no prompt logits");
    }

    let mut generated_ids = Vec::with_capacity(max_tokens);
    let mut decoded_len = 0;
    for _ in 0..max_tokens {
        let next = crate::sampling::sample_next(&logits, sampling, &mut rng);
        generated_ids.push(next);

        let full_ids: Vec<u32> = generated_ids.iter().map(|&id| id as u32).collect();
        let full_text = worker
            .0
            .tokenizer
            .decode(&full_ids, true)
            .unwrap_or_default();
        let hit_eos = model.eos_token_id() == Some(next);
        let hit_limit = generated_ids.len() >= max_tokens;
        let force_flush = hit_eos || hit_limit;
        let (visible_end, stopped_by_sequence) =
            stop_limited_visible_end(&full_text, &item.stop_sequences, force_flush);
        let visible_text = full_text[..visible_end].to_string();
        let delta = if visible_end > decoded_len {
            full_text[decoded_len..visible_end].to_string()
        } else {
            String::new()
        };
        decoded_len = visible_end.max(decoded_len);

        if !delta.is_empty() {
            let msg = WorkerMessage::Delta { text: delta };
            if item
                .response_tx
                .blocking_send(serialize_channel_message(&msg))
                .is_err()
            {
                return Ok(generated_ids.len());
            }
        }

        let finished = stopped_by_sequence || hit_eos || hit_limit;
        if finished {
            let finish_reason = if stopped_by_sequence || hit_eos {
                "stop"
            } else {
                "length"
            }
            .to_string();
            let total_completion = generated_ids.len();
            let plan = ZymaticaPlan {
                request_id: item.request_id,
                prompt_tokens: item.prompt_ids.len(),
                reusable_prefix_tokens: 0,
                billable_tokens: item.prompt_ids.len(),
                scheduler_total_billable_tokens: item.prompt_ids.len() + total_completion,
            };
            let done_msg = WorkerMessage::Done {
                text: visible_text,
                prompt_tokens: item.prompt_ids.len(),
                completion_tokens: total_completion,
                finish_reason,
                plan,
            };
            let _ = item
                .response_tx
                .blocking_send(serialize_channel_message(&done_msg));
            return Ok(total_completion);
        }

        let pos = item.prompt_ids.len() + generated_ids.len() - 1;
        logits = model.forward_token(next, pos, &mut cache);
    }

    Ok(generated_ids.len())
}

fn run_continuous_batcher(
    worker: ModelWorkerHandle,
    mut request_rx: tokio::sync::mpsc::Receiver<QueueItem>,
) -> Result<()> {
    let layer_shapes = worker.0.model.layer_shapes();
    let mut kv_cache = crate::paged_kv::PagedKvCache::new_with_shapes(&layer_shapes, 8);
    let mut active_requests: Vec<ActiveRequest> = Vec::new();
    let mut scheduler_iteration = 0_u64;

    loop {
        scheduler_iteration = scheduler_iteration.saturating_add(1);
        if active_requests.is_empty() {
            if let Some(item) = request_rx.blocking_recv() {
                let response_tx = item.response_tx.clone();
                match init_active_request(item, &worker) {
                    Ok(req) => active_requests.push(req),
                    Err(e) => {
                        let _ = response_tx
                            .blocking_send(Err(format!("Error initializing request: {e}")));
                        worker.0.active_requests.fetch_sub(1, Ordering::Relaxed);
                    }
                }
            } else {
                break; // queue closed
            }
        }

        while let Ok(item) = request_rx.try_recv() {
            let response_tx = item.response_tx.clone();
            match init_active_request(item, &worker) {
                Ok(req) => active_requests.push(req),
                Err(e) => {
                    let _ =
                        response_tx.blocking_send(Err(format!("Error initializing request: {e}")));
                    worker.0.active_requests.fetch_sub(1, Ordering::Relaxed);
                }
            }
        }

        active_requests.retain(|req| {
            let active = !req.response_tx.is_closed();
            if !active {
                remove_swap_file(req.kv_swapped_path.as_deref());
                kv_cache.free_sequence(req.id);
                worker.0.active_requests.fetch_sub(1, Ordering::Relaxed);
            }
            active
        });

        if active_requests.is_empty() {
            continue;
        }

        // Prefill step: advance one waiting request by a bounded chunk, then return to decode.
        if let Some(req) = active_requests.iter_mut().find(|r| !r.prefill_done) {
            let chunk_start = Instant::now();
            ensure_request_resident(&mut kv_cache, req)?;

            let prompt_len = req.prompt_ids.len();
            if req.prefill_pos == 0 {
                let (mut reusable_tokens, mut shared_pages) =
                    if let Ok(scheduler) = worker.0.scheduler.lock() {
                        let candidate = scheduler.prefix_cache.longest_match(&req.prompt_ids).map(
                            |(len, val)| {
                                (len, val.cache_pages.clone(), val.page_generations.clone())
                            },
                        );
                        if let Some((len, pages, generations)) = candidate {
                            if kv_cache.validate_page_handles(&pages, &generations) {
                                (len, pages)
                            } else {
                                (0, Vec::new())
                            }
                        } else {
                            (0, Vec::new())
                        }
                    } else {
                        (0, Vec::new())
                    };

                if prompt_len > 0 && reusable_tokens >= prompt_len {
                    reusable_tokens = prompt_len - 1;
                    shared_pages.truncate(reusable_tokens.div_ceil(kv_cache.page_size));
                }

                kv_cache.create_sequence_with_pages(req.id, &shared_pages, reusable_tokens);
                req.reusable_prefix_tokens = reusable_tokens;
                req.prefill_pos = reusable_tokens;
            }

            let mut logits = Vec::new();
            let start_pos = req.prefill_pos;
            let end_pos = prompt_len.min(start_pos + worker.0.prefill_chunk_tokens);

            if start_pos < end_pos {
                let cache_ptr = crate::model::SharedPagedKvCache(&mut kv_cache as *mut _);
                let mut cache_wrapper = AnyKvCache::Paged {
                    cache: cache_ptr,
                    sequence_id: req.id,
                };
                for pos in start_pos..end_pos {
                    let token_id = req.prompt_ids[pos];
                    logits = worker
                        .0
                        .model
                        .forward_token(token_id, pos, &mut cache_wrapper);
                }
            }

            req.prefill_pos = end_pos;
            if !logits.is_empty() {
                req.last_logits = logits;
            }
            req.metrics.prefill_time_ms += chunk_start.elapsed().as_millis() as u64;
            req.last_touched_iteration = scheduler_iteration;
            publish_completed_prefix_pages(&worker, &kv_cache, req);

            if let Some(draft_model) = worker.0.draft_model.as_ref()
                && let Some(draft_cache) = req.draft_cache.as_mut()
            {
                let draft_end =
                    prompt_len.min(req.draft_prefill_pos + worker.0.prefill_chunk_tokens);
                let mut draft_logits = Vec::new();
                for pos in req.draft_prefill_pos..draft_end {
                    let token_id = req.prompt_ids[pos];
                    draft_logits = draft_model.forward_token(token_id, pos, draft_cache);
                }
                req.draft_prefill_pos = draft_end;
                if !draft_logits.is_empty() {
                    req.draft_last_logits = draft_logits;
                }
            }

            let draft_ready = req.draft_cache.is_none() || req.draft_prefill_pos >= prompt_len;
            if req.prefill_pos >= prompt_len && draft_ready {
                req.prefill_done = true;
                req.metrics.queue_time_ms = req.created_time.elapsed().as_millis() as u64;
            }
        }

        let mut completed = Vec::new();
        let mut decode_indices = Vec::new();
        let mut decode_batch = Vec::new();
        let mut decode_caches = Vec::new();

        for (idx, req) in active_requests.iter_mut().enumerate() {
            if !req.prefill_done {
                continue;
            }

            if let Err(e) = ensure_request_resident(&mut kv_cache, req) {
                let msg = WorkerMessage::Error {
                    message: format!("Failed to restore KV cache: {e}"),
                };
                let _ = req
                    .response_tx
                    .blocking_send(serialize_channel_message(&msg));
                completed.push(idx);
                continue;
            }

            req.last_touched_iteration = scheduler_iteration;

            if can_use_speculative(req, &worker)
                && let Some(draft_model) = worker.0.draft_model.as_ref()
            {
                if run_speculative_step(&worker, &mut kv_cache, draft_model, req)? {
                    completed.push(idx);
                }
                continue;
            }

            if can_use_ngram_speculative(req, &worker)
                && let Some(finished) = run_ngram_speculative_step(&worker, &mut kv_cache, req)?
            {
                if finished {
                    completed.push(idx);
                }
                continue;
            }

            let step_start = Instant::now();
            let next = crate::sampling::sample_next(&req.last_logits, req.sampling, &mut req.rng);
            let finished = emit_generated_token(&worker, req, next, step_start);
            if finished {
                completed.push(idx);
            } else {
                let pos = req.prompt_ids.len() + req.generated_ids.len() - 1;
                decode_indices.push(idx);
                decode_batch.push((next, pos));
                let cache_ptr = crate::model::SharedPagedKvCache(&mut kv_cache as *mut _);
                decode_caches.push(AnyKvCache::Paged {
                    cache: cache_ptr,
                    sequence_id: req.id,
                });
            }
        }

        if !decode_batch.is_empty() {
            let batch_logits = worker
                .0
                .model
                .forward_batch(&decode_batch, &mut decode_caches);
            for (i, &idx) in decode_indices.iter().enumerate() {
                active_requests[idx].last_logits = batch_logits[i].clone();
            }
        }

        for &idx in completed.iter().rev() {
            let req = &active_requests[idx];
            remove_swap_file(req.kv_swapped_path.as_deref());
            kv_cache.free_sequence(req.id);
            worker.0.active_requests.fetch_sub(1, Ordering::Relaxed);
            worker
                .0
                .prompt_tokens
                .fetch_add(req.prompt_ids.len() as u64, Ordering::Relaxed);
            worker
                .0
                .completion_tokens
                .fetch_add(req.generated_ids.len() as u64, Ordering::Relaxed);
            active_requests.remove(idx);
        }

        enforce_kv_swap_policy(&worker, &mut kv_cache, &mut active_requests);
        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn can_use_speculative(req: &ActiveRequest, worker: &ModelWorkerHandle) -> bool {
    worker.0.draft_model.is_some()
        && worker.0.draft_k > 0
        && req.draft_cache.is_some()
        && req.draft_controller.is_some()
        && req.sampling.temperature <= 0.0
        && req.sampling.top_k <= 1
        && !req.last_logits.is_empty()
        && !req.draft_last_logits.is_empty()
}

fn can_use_ngram_speculative(req: &ActiveRequest, worker: &ModelWorkerHandle) -> bool {
    worker.0.draft_model.is_none()
        && worker.0.draft_k > 0
        && req.ngram_proposer.is_some()
        && req.draft_controller.is_some()
        && req.sampling.temperature <= 0.0
        && req.sampling.top_k <= 1
        && !req.last_logits.is_empty()
}

fn recent_token_context(
    prompt_ids: &[usize],
    generated_ids: &[usize],
    context_len: usize,
) -> Vec<usize> {
    let total_len = prompt_ids.len() + generated_ids.len();
    let start = total_len.saturating_sub(context_len);
    (start..total_len)
        .map(|index| {
            if index < prompt_ids.len() {
                prompt_ids[index]
            } else {
                generated_ids[index - prompt_ids.len()]
            }
        })
        .collect()
}

fn run_ngram_speculative_step(
    worker: &ModelWorkerHandle,
    kv_cache: &mut crate::paged_kv::PagedKvCache,
    req: &mut ActiveRequest,
) -> Result<Option<bool>> {
    let max_tok = max_tokens_for_request(worker, req);
    let remaining = max_tok.saturating_sub(req.generated_ids.len());
    if remaining == 0 {
        return Ok(Some(true));
    }

    let step_k = req
        .draft_controller
        .as_ref()
        .map(|controller| controller.current_k())
        .unwrap_or(1)
        .min(remaining)
        .max(1);
    let candidates = {
        let proposer = req
            .ngram_proposer
            .as_ref()
            .context("n-gram speculative request missing proposer")?;
        let context =
            recent_token_context(&req.prompt_ids, &req.generated_ids, proposer.context_len());
        proposer.propose_sequence(&context, step_k)
    };
    if candidates.is_empty() {
        return Ok(None);
    }
    worker
        .0
        .ngram_proposal_steps
        .fetch_add(1, Ordering::Relaxed);
    worker
        .0
        .ngram_proposed_tokens
        .fetch_add(candidates.len() as u64, Ordering::Relaxed);

    let step_start = Instant::now();
    let verification_start = req.prompt_ids.len() + req.generated_ids.len();
    let block_logits =
        worker
            .0
            .model
            .forward_candidate_block(req.id, verification_start, &candidates, kv_cache);

    let mut accepted = 0_usize;
    let mut expected = crate::ops::argmax(&req.last_logits);
    for (idx, &candidate) in candidates.iter().enumerate() {
        if expected != candidate {
            break;
        }
        accepted += 1;
        if idx < block_logits.len() {
            expected = crate::ops::argmax(&block_logits[idx]);
        }
    }
    worker
        .0
        .ngram_accepted_tokens
        .fetch_add(accepted as u64, Ordering::Relaxed);
    kv_cache.truncate_sequence(req.id, verification_start + accepted)?;

    if let Some(controller) = req.draft_controller.as_mut() {
        controller.observe(accepted, candidates.len());
    }

    for &candidate in candidates.iter().take(accepted) {
        if emit_generated_token(worker, req, candidate, step_start) {
            return Ok(Some(true));
        }
    }

    if accepted > 0 {
        req.last_logits = block_logits[accepted - 1].clone();
    }
    if req.generated_ids.len() >= max_tok {
        return Ok(Some(true));
    }

    let next_token = if accepted == candidates.len() {
        crate::ops::argmax(&block_logits[candidates.len() - 1])
    } else {
        crate::ops::argmax(&req.last_logits)
    };
    let pos = req.prompt_ids.len() + req.generated_ids.len();
    if emit_generated_token(worker, req, next_token, step_start) {
        return Ok(Some(true));
    }

    let cache_ptr = crate::model::SharedPagedKvCache(kv_cache as *mut _);
    let mut target_cache = AnyKvCache::Paged {
        cache: cache_ptr,
        sequence_id: req.id,
    };
    req.last_logits = worker
        .0
        .model
        .forward_token(next_token, pos, &mut target_cache);
    Ok(Some(false))
}

fn run_speculative_step(
    worker: &ModelWorkerHandle,
    kv_cache: &mut crate::paged_kv::PagedKvCache,
    draft_model: &RuntimeModel,
    req: &mut ActiveRequest,
) -> Result<bool> {
    let max_tok = max_tokens_for_request(worker, req);
    let remaining = max_tok.saturating_sub(req.generated_ids.len());
    if remaining == 0 {
        return Ok(true);
    }

    let step_k = req
        .draft_controller
        .as_ref()
        .map(|controller| controller.current_k())
        .unwrap_or(1)
        .min(remaining)
        .max(1);

    let mut candidates = Vec::with_capacity(step_k);
    let mut temp_draft_logits = req.draft_last_logits.clone();
    let mut temp_draft_cache = req
        .draft_cache
        .as_ref()
        .context("speculative request missing draft cache")?
        .clone();
    let base_generated = req.generated_ids.len();
    for i in 0..step_k {
        let next_draft = crate::ops::argmax(&temp_draft_logits);
        candidates.push(next_draft);
        let pos = req.prompt_ids.len() + base_generated + i;
        temp_draft_logits = draft_model.forward_token(next_draft, pos, &mut temp_draft_cache);
    }

    let verification_start = req.prompt_ids.len() + req.generated_ids.len();
    let block_logits =
        worker
            .0
            .model
            .forward_candidate_block(req.id, verification_start, &candidates, kv_cache);

    let mut accepted = 0_usize;
    let mut expected = crate::ops::argmax(&req.last_logits);
    for (idx, &candidate) in candidates.iter().enumerate() {
        if expected != candidate {
            break;
        }
        accepted += 1;
        if idx < block_logits.len() {
            expected = crate::ops::argmax(&block_logits[idx]);
        }
    }

    kv_cache.truncate_sequence(req.id, verification_start + accepted)?;

    if let Some(controller) = req.draft_controller.as_mut() {
        controller.observe(accepted, candidates.len());
    }

    for &candidate in candidates.iter().take(accepted) {
        let finished = emit_generated_token(worker, req, candidate, Instant::now());
        if let Some(draft_cache) = req.draft_cache.as_mut() {
            let pos = req.prompt_ids.len() + req.generated_ids.len() - 1;
            req.draft_last_logits = draft_model.forward_token(candidate, pos, draft_cache);
        }
        if finished {
            return Ok(true);
        }
    }

    if accepted > 0 {
        req.last_logits = block_logits[accepted - 1].clone();
    }
    if req.generated_ids.len() >= max_tok {
        return Ok(true);
    }

    let next_token = if accepted == candidates.len() {
        crate::ops::argmax(&block_logits[candidates.len() - 1])
    } else if accepted == 0 {
        crate::ops::argmax(&req.last_logits)
    } else {
        crate::ops::argmax(&block_logits[accepted - 1])
    };
    let pos = req.prompt_ids.len() + req.generated_ids.len();
    let finished = emit_generated_token(worker, req, next_token, Instant::now());
    if finished {
        return Ok(true);
    }

    let cache_ptr = crate::model::SharedPagedKvCache(kv_cache as *mut _);
    let mut target_cache = AnyKvCache::Paged {
        cache: cache_ptr,
        sequence_id: req.id,
    };
    req.last_logits = worker
        .0
        .model
        .forward_token(next_token, pos, &mut target_cache);
    if let Some(draft_cache) = req.draft_cache.as_mut() {
        req.draft_last_logits = draft_model.forward_token(next_token, pos, draft_cache);
    }
    Ok(false)
}

fn stop_limited_visible_end(
    text: &str,
    stop_sequences: &[String],
    force_flush: bool,
) -> (usize, bool) {
    if stop_sequences.is_empty() {
        return (text.len(), false);
    }

    if let Some(stop_start) = stop_sequences
        .iter()
        .filter_map(|sequence| text.find(sequence))
        .min()
    {
        return (stop_start, true);
    }

    if force_flush {
        return (text.len(), false);
    }

    let held_prefix_bytes = pending_stop_prefix_bytes(text, stop_sequences);
    (text.len().saturating_sub(held_prefix_bytes), false)
}

fn pending_stop_prefix_bytes(text: &str, stop_sequences: &[String]) -> usize {
    let mut max_prefix = 0;
    for sequence in stop_sequences {
        for (prefix_len, _) in sequence.char_indices().skip(1) {
            if text.ends_with(&sequence[..prefix_len]) {
                max_prefix = max_prefix.max(prefix_len);
            }
        }
    }
    max_prefix
}

fn emit_generated_token(
    worker: &ModelWorkerHandle,
    req: &mut ActiveRequest,
    token: usize,
    step_start: Instant,
) -> bool {
    if let Some(context_len) = req
        .ngram_proposer
        .as_ref()
        .map(FastNGramProposalEngine::context_len)
    {
        let context = recent_token_context(&req.prompt_ids, &req.generated_ids, context_len);
        if let Some(proposer) = req.ngram_proposer.as_mut() {
            proposer.observe_transition(&context, token);
        }
    }
    req.generated_ids.push(token);

    let full_ids: Vec<u32> = req.generated_ids.iter().map(|&id| id as u32).collect();
    let full_text = worker
        .0
        .tokenizer
        .decode(&full_ids, true)
        .unwrap_or_default();
    let hit_eos = token == 2;
    let hit_limit = req.generated_ids.len() >= max_tokens_for_request(worker, req);
    let force_flush = hit_eos || hit_limit;
    let (visible_end, stopped_by_sequence) =
        stop_limited_visible_end(&full_text, &req.stop_sequences, force_flush);
    let visible_text = full_text[..visible_end].to_string();
    let delta = if visible_end > req.decoded_len {
        full_text[req.decoded_len..visible_end].to_string()
    } else {
        String::new()
    };
    req.decoded_len = visible_end.max(req.decoded_len);

    if !delta.is_empty() {
        let msg = WorkerMessage::Delta { text: delta };
        let _ = req
            .response_tx
            .blocking_send(serialize_channel_message(&msg));
    }

    req.metrics
        .token_latencies_ms
        .push(step_start.elapsed().as_millis() as u64);

    let finished = stopped_by_sequence || hit_eos || hit_limit;
    if finished {
        let total_completion = req.generated_ids.len();
        let finish_reason = if stopped_by_sequence || hit_eos {
            "stop"
        } else {
            "length"
        }
        .to_string();
        let plan = ZymaticaPlan {
            request_id: req.id,
            prompt_tokens: req.prompt_ids.len(),
            reusable_prefix_tokens: req.reusable_prefix_tokens,
            billable_tokens: req
                .prompt_ids
                .len()
                .saturating_sub(req.reusable_prefix_tokens),
            scheduler_total_billable_tokens: req
                .prompt_ids
                .len()
                .saturating_sub(req.reusable_prefix_tokens)
                + total_completion,
        };
        let done_msg = WorkerMessage::Done {
            text: visible_text,
            prompt_tokens: req.prompt_ids.len(),
            completion_tokens: total_completion,
            finish_reason,
            plan,
        };
        let _ = req
            .response_tx
            .blocking_send(serialize_channel_message(&done_msg));
    }
    finished
}

fn max_tokens_for_request(worker: &ModelWorkerHandle, req: &ActiveRequest) -> usize {
    req.max_new_tokens.clamp(1, worker.0.max_new_tokens.max(1))
}

fn ensure_request_resident(
    kv_cache: &mut crate::paged_kv::PagedKvCache,
    req: &mut ActiveRequest,
) -> Result<()> {
    let Some(path) = req.kv_swapped_path.take() else {
        return Ok(());
    };
    kv_cache
        .restore_sequence_from_path(&path)
        .with_context(|| {
            format!(
                "restoring swapped KV sequence {} from {}",
                req.id,
                path.display()
            )
        })?;
    remove_swap_file(Some(&path));
    Ok(())
}

fn enforce_kv_swap_policy(
    worker: &ModelWorkerHandle,
    kv_cache: &mut crate::paged_kv::PagedKvCache,
    active_requests: &mut [ActiveRequest],
) {
    let Some(swap_dir) = worker.0.kv_swap_dir.as_ref() else {
        return;
    };
    let max_pages = worker.0.kv_max_resident_pages;
    if max_pages == 0 {
        return;
    }
    let threshold = worker.0.kv_swap_threshold.clamp(0.01, 1.0);
    let threshold_pages = ((max_pages as f32) * threshold).ceil().max(1.0) as usize;
    if kv_cache.resident_pages() <= threshold_pages {
        return;
    }

    let resident_candidates = active_requests
        .iter()
        .filter(|req| req.prefill_done && req.kv_swapped_path.is_none())
        .count();
    if resident_candidates <= 1 {
        return;
    }

    let Some(idx) = active_requests
        .iter()
        .enumerate()
        .filter(|(_, req)| {
            req.prefill_done && req.kv_swapped_path.is_none() && !req.response_tx.is_closed()
        })
        .min_by(|(_, a), (_, b)| {
            let a_energy = kv_cache
                .sequence_mean_l2_energy(a.id)
                .unwrap_or(f32::INFINITY);
            let b_energy = kv_cache
                .sequence_mean_l2_energy(b.id)
                .unwrap_or(f32::INFINITY);
            a_energy
                .total_cmp(&b_energy)
                .then_with(|| a.last_touched_iteration.cmp(&b.last_touched_iteration))
        })
        .map(|(idx, _)| idx)
    else {
        return;
    };

    let req = &mut active_requests[idx];
    req.kv_swap_count = req.kv_swap_count.saturating_add(1);
    let path = swap_dir.join(format!("request-{}-{}.zkv", req.id, req.kv_swap_count));
    match kv_cache.swap_out_sequence_to_path(req.id, &path) {
        Ok(_) => {
            req.kv_swapped_path = Some(path);
        }
        Err(e) => {
            eprintln!("KV swap-out failed for request {}: {e}", req.id);
        }
    }
}

fn remove_swap_file(path: Option<&Path>) {
    if let Some(path) = path
        && let Err(e) = fs::remove_file(path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!("failed to remove KV swap file {}: {e}", path.display());
    }
}

fn publish_completed_prefix_pages(
    worker: &ModelWorkerHandle,
    kv_cache: &crate::paged_kv::PagedKvCache,
    req: &ActiveRequest,
) {
    let page_handles = kv_cache.get_page_handles(req.id);
    if page_handles.is_empty() {
        return;
    }

    if let Ok(mut scheduler) = worker.0.scheduler.lock() {
        let page_size = kv_cache.page_size;
        for p in 1..=page_handles.len() {
            let prefix_len = p * page_size;
            if prefix_len <= req.prefill_pos && prefix_len <= req.prompt_ids.len() {
                let sub_tokens = &req.prompt_ids[..prefix_len];
                let sub_pages = page_handles[..p].to_vec();
                let sub_generations = kv_cache.get_page_generations(&sub_pages);
                scheduler.prefix_cache.insert(
                    sub_tokens,
                    PrefixValue {
                        cache_pages: sub_pages,
                        page_generations: sub_generations,
                        token_len: prefix_len,
                    },
                );
            }
        }
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn api_error(status: StatusCode, error: impl Into<String>) -> ApiError {
    (
        status,
        Json(ErrorResponse {
            error: error.into(),
        }),
    )
}

fn internal_error(error: impl std::fmt::Display) -> ApiError {
    api_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn serialize_channel_message<T: Serialize>(message: &T) -> std::result::Result<String, String> {
    serde_json::to_string(message).map_err(|error| format!("serializing server message: {error}"))
}

fn check_auth(headers: &axum::http::HeaderMap) -> Result<(), ApiError> {
    if let Ok(key) = std::env::var("ZYMATICA_API_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            let authorized = headers
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .map(|auth_str| {
                    auth_str.starts_with("Bearer ") && auth_str["Bearer ".len()..].trim() == key
                })
                .unwrap_or(false);
            if authorized {
                return Ok(());
            }
            return Err(api_error(
                StatusCode::UNAUTHORIZED,
                "Unauthorized: Invalid or missing API key",
            ));
        }
    }
    Ok(())
}

fn resolve_model_worker(
    state: &ServerStateHandle,
    requested_model: Option<&str>,
) -> Result<ModelWorkerHandle, ApiError> {
    let requested_model = requested_model.unwrap_or(&state.0.default_model_name);
    if requested_model.contains('/')
        || requested_model.contains('\\')
        || requested_model.contains("..")
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "Access denied: invalid model name/path",
        ));
    }
    if let Some(worker) = state.0.models.get(requested_model) {
        return Ok(worker.clone());
    }
    if state.0.default_model_name.ends_with(requested_model)
        && let Some(worker) = state.0.models.get(&state.0.default_model_name)
    {
        return Ok(worker.clone());
    }
    Err(api_error(
        StatusCode::BAD_REQUEST,
        format!(
            "Model '{}' not loaded by this server instance",
            requested_model
        ),
    ))
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
    id: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

async fn handle_mcp(
    State(state): State<ServerStateHandle>,
    Json(payload): Json<JsonRpcRequest>,
) -> (StatusCode, Json<JsonRpcResponse>) {
    if payload.jsonrpc != "2.0" {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32600,
                    message: "Invalid Request: expected jsonrpc version '2.0'".to_string(),
                }),
                id: payload.id,
            }),
        );
    }

    match payload.method.as_str() {
        "tools/list" => {
            let tools = serde_json::json!({
                "tools": [
                    {
                        "name": "get_system_telemetry",
                        "description": "Returns system metrics, device thermals, memory swapping rates, and hardware load profile.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "list_registry_models",
                        "description": "Lists all model weights currently cached or registered on the Zymatica-Engine node.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "execute_local_rag",
                        "description": "Performs local context retrieval and paragraph-matching term search against a specified directory.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "The search term or query"
                                },
                                "docs_dir": {
                                    "type": "string",
                                    "description": "Optional search directory path"
                                }
                            },
                            "required": ["query"]
                        }
                    }
                ]
            });
            (
                StatusCode::OK,
                Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(tools),
                    error: None,
                    id: payload.id,
                }),
            )
        }
        "tools/call" => {
            let name = payload.params.get("name").and_then(|v| v.as_str());
            let arguments = payload
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            let name = match name {
                Some(n) => n,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Invalid Params: missing tool name".to_string(),
                            }),
                            id: payload.id,
                        }),
                    );
                }
            };

            let response_val = match name {
                "get_system_telemetry" => {
                    let mut num_reqs = 0;
                    let mut active_reqs = 0;
                    let mut p_tokens = 0;
                    let mut c_tokens = 0;
                    if let Some(worker) = state.0.models.get(&state.0.default_model_name) {
                        num_reqs = worker.0.num_requests.load(Ordering::Relaxed);
                        active_reqs = worker.0.active_requests.load(Ordering::Relaxed);
                        p_tokens = worker.0.prompt_tokens.load(Ordering::Relaxed);
                        c_tokens = worker.0.completion_tokens.load(Ordering::Relaxed);
                    }
                    let thermal = std::env::var("ZYMATICA_THERMAL_PRESSURE")
                        .unwrap_or_else(|_| "normal".to_string());
                    serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!(
                                    "System Telemetry:\n- Thermal Pressure: {}\n- Total Requests: {}\n- Active Requests: {}\n- Prompt Tokens: {}\n- Completion Tokens: {}",
                                    thermal, num_reqs, active_reqs, p_tokens, c_tokens
                                )
                            }
                        ]
                    })
                }
                "list_registry_models" => {
                    let model_names: Vec<String> = state.0.models.keys().cloned().collect();
                    serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Registered Models: {:?}", model_names)
                            }
                        ]
                    })
                }
                "execute_local_rag" => {
                    let query = arguments
                        .get("query")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let docs_dir = arguments.get("docs_dir").and_then(|v| v.as_str());
                    let docs_path = docs_dir.map(Path::new);

                    match handle_local_rag(query, docs_path) {
                        Ok(text) => {
                            serde_json::json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": text
                                    }
                                ]
                            })
                        }
                        Err(e) => {
                            serde_json::json!({
                                "isError": true,
                                "content": [
                                    {
                                        "type": "text",
                                        "text": format!("RAG execution failed: {e}")
                                    }
                                ]
                            })
                        }
                    }
                }
                other => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32601,
                                message: format!(
                                    "Method Not Found: tool '{}' does not exist",
                                    other
                                ),
                            }),
                            id: payload.id,
                        }),
                    );
                }
            };

            (
                StatusCode::OK,
                Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(response_val),
                    error: None,
                    id: payload.id,
                }),
            )
        }
        other => (
            StatusCode::NOT_FOUND,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method Not Found: '{}'", other),
                }),
                id: payload.id,
            }),
        ),
    }
}

fn handle_local_rag(query: &str, docs_dir: Option<&Path>) -> Result<String> {
    let mut chunks = Vec::new();
    if let Some(dir) = docs_dir.filter(|d| d.exists()) {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Ok(content) = std::fs::read_to_string(&path)
            {
                for paragraph in content.split("\n\n") {
                    let clean = paragraph.trim();
                    if !clean.is_empty() {
                        chunks.push(clean.to_string());
                    }
                }
            }
        }
    }

    if chunks.is_empty() {
        chunks.push("Field status: Solar array power output is currently normal at 8.4 kW. Grid load is 5.2 kW.".to_string());
        chunks.push("Water levels at Section A-4 reservoir are currently at 84% capacity. Flow rate: 1.2 m3/s.".to_string());
        chunks.push(
            "The capital of France is Paris. Paris is known for its cafes and history.".to_string(),
        );
    }

    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let mut best_chunk = &chunks[0];
    let mut best_score = 0;

    for chunk in &chunks {
        let chunk_lower = chunk.to_lowercase();
        let mut score = 0;
        for &word in &query_words {
            if word.len() > 3 && chunk_lower.contains(word) {
                score += 1;
            }
        }
        if score > best_score {
            best_score = score;
            best_chunk = chunk;
        }
    }

    Ok(best_chunk.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ngram_context_crosses_prompt_generation_boundary() {
        assert_eq!(recent_token_context(&[1, 2, 3], &[4], 3), vec![2, 3, 4]);
        assert_eq!(recent_token_context(&[1, 2], &[], 4), vec![1, 2]);
    }

    #[test]
    fn completion_request_accepts_prompt_token_ids_and_max_completion_tokens() {
        let request: CompletionRequest = serde_json::from_value(serde_json::json!({
            "model": "zymatica-q8",
            "prompt": [1, 2, 3],
            "max_completion_tokens": 7,
            "stop": ["\n", "###"]
        }))
        .unwrap();

        match request.prompt.unwrap() {
            PromptInput::Tokens(ids) => assert_eq!(ids, vec![1, 2, 3]),
            _ => panic!("expected token-id prompt"),
        }
        assert_eq!(
            resolve_max_tokens(request.max_tokens, request.max_completion_tokens).unwrap(),
            7
        );
        assert_eq!(normalize_stop_sequences(request.stop).unwrap().len(), 2);
    }

    #[test]
    fn chat_content_accepts_openai_text_parts() {
        let message: ChatMessage = serde_json::from_value(serde_json::json!({
            "role": "user",
            "content": [
                { "type": "text", "text": "hello " },
                { "type": "input_text", "text": "world" }
            ]
        }))
        .unwrap();

        assert_eq!(message.content.to_text().unwrap(), "hello world");
    }

    #[test]
    fn max_token_alias_rejects_disagreement() {
        assert!(resolve_max_tokens(Some(4), Some(5)).is_err());
        assert_eq!(resolve_max_tokens(Some(4), Some(4)).unwrap(), 4);
        assert_eq!(resolve_max_tokens(None, Some(6)).unwrap(), 6);
    }

    #[test]
    fn stop_window_holds_partial_prefix_and_truncates_match() {
        let stops = vec!["STOP".to_string()];

        assert_eq!(
            stop_limited_visible_end("hello ST", &stops, false),
            (6, false)
        );
        assert_eq!(
            stop_limited_visible_end("hello STOP after", &stops, false),
            (6, true)
        );
        assert_eq!(
            stop_limited_visible_end("hello ST", &stops, true),
            (8, false)
        );
    }

    #[tokio::test]
    async fn test_mcp_tools_list_direct() {
        let models = Arc::new(HashMap::new());
        let state = ServerStateHandle(Arc::new(ServerStateInner {
            default_model_name: "test-model".to_string(),
            models,
            next_request_id: Arc::new(AtomicU64::new(1)),
        }));

        let payload = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: serde_json::Value::Null,
            id: serde_json::json!(1),
        };

        let (status, Json(response)) = handle_mcp(State(state), Json(payload)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result_val = response.result.unwrap();
        let tools = result_val.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 3);
        assert_eq!(
            tools[0].get("name").unwrap().as_str().unwrap(),
            "get_system_telemetry"
        );
        assert_eq!(
            tools[1].get("name").unwrap().as_str().unwrap(),
            "list_registry_models"
        );
        assert_eq!(
            tools[2].get("name").unwrap().as_str().unwrap(),
            "execute_local_rag"
        );
    }

    #[tokio::test]
    async fn test_mcp_post_route_tools_list() {
        use axum::body::{Body, to_bytes};
        use axum::http::Request;
        use tower::ServiceExt;

        let models = Arc::new(HashMap::new());
        let state = ServerStateHandle(Arc::new(ServerStateInner {
            default_model_name: "test-model".to_string(),
            models,
            next_request_id: Arc::new(AtomicU64::new(1)),
        }));
        let app = Router::new()
            .route("/mcp", post(handle_mcp))
            .with_state(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "tools/list",
                            "params": {},
                            "id": 11
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), MAX_REQUEST_BODY_BYTES)
            .await
            .unwrap();
        let response: JsonRpcResponse = serde_json::from_slice(&bytes).unwrap();
        let tools = response
            .result
            .unwrap()
            .get("tools")
            .unwrap()
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(tools.len(), 3);
    }

    #[tokio::test]
    async fn test_mcp_tools_call_telemetry_and_rag() {
        let models = Arc::new(HashMap::new());
        let state = ServerStateHandle(Arc::new(ServerStateInner {
            default_model_name: "test-model".to_string(),
            models,
            next_request_id: Arc::new(AtomicU64::new(1)),
        }));

        // Test telemetry call
        let payload_telemetry = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "get_system_telemetry"
            }),
            id: serde_json::json!(2),
        };

        let (status, Json(response)) =
            handle_mcp(State(state.clone()), Json(payload_telemetry)).await;
        assert_eq!(status, StatusCode::OK);
        let result_val = response.result.unwrap();
        let content = result_val.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("System Telemetry"));

        // Test RAG call
        let payload_rag = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "execute_local_rag",
                "arguments": {
                    "query": "What is the capital of France?"
                }
            }),
            id: serde_json::json!(3),
        };

        let (status, Json(response)) = handle_mcp(State(state), Json(payload_rag)).await;
        assert_eq!(status, StatusCode::OK);
        let result_val = response.result.unwrap();
        let content = result_val.get("content").unwrap().as_array().unwrap();
        let text = content[0].get("text").unwrap().as_str().unwrap();
        assert!(text.contains("Paris"));
    }
}
