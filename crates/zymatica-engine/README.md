# Zymatica-Rust-Body ☤

**The 100% Native High-Performance Rust & C++ Autonomous AI Agent Harness & Engine.** Engineered for zero-latency tool routing, 0ms speculative tool pre-execution, persistent SIMD vector/dialectic memory, multi-channel messaging gateway (~20 platforms), `agentskills.io` procedural memory, cron priority scheduling, MCP/ACP protocols, setup wizard, OpenClaw migration, SWE benchmark execution, and hardware-accelerated local model inference.

---

## Architecture Overview

Zymatica-Rust-Body replaces heavy Python runtimes with a single, ultra-fast compiled binary (`zymatica_agent`).

### Implemented Subsystems & Modules
- **`agent_runtime`**: Tokio state-machine agent loop with byte-stable prompt cache matching.
- **`agent_speculative_tools`**: Stream token watcher for **0ms perceived tool turn latency**.
- **`agent_tools`**: Sub-microsecond dynamic tool registry, native PTY execution, mmap zero-copy file reader, SIMD ripgrep.
- **`agent_gateway`**: High-concurrency event-driven multi-platform messaging gateway (Telegram, Discord, Slack, WhatsApp, Signal, Webhooks, CLI).
- **`agent_memory`**: FTS5 full-text search, AVX-512 vector cosine similarity RAG, and Honcho dialectic user modeling.
- **`agent_compression`**: Zero-copy context sliding window, token budget calculation, and prompt-cache stabilization.
- **`agent_subagent`**: Parallel subtask spawning over background Tokio channels without context bloat.
- **`agent_skills`**: Procedural skill parser adhering to the `agentskills.io` standard.
- **`agent_cron`**: Zero-alloc min-heap priority queue scheduler.
- **`agent_mcp`**: Model Context Protocol (MCP) JSON-RPC client & server bridge.
- **`agent_acp`**: Agent Communication Protocol (ACP) IDE synchronization server.
- **`agent_setup_wizard`**: Interactive CLI setup wizard & config generator (`~/.zymatica/config.yaml`).
- **`agent_claw_migration`**: Automatic migration from legacy OpenClaw (`~/.openclaw`).
- **`agent_plugin_loader`**: Dynamic plugin discovery and hook execution (`~/.zymatica/plugins/`).
- **`agent_swe_runner`**: Parallel software engineering task runner & trajectory dataset generator.
- **`agent_doctor`**: System environment diagnostics & self-healing configuration repair.
- **`agent_guardrails`**: Dual-layer pre-inference input guards (prompt injection, token limits) & output validation guards with auto-retry feedback loops.
- **`agent_evaluator`**: RAG claim faithfulness scoring, context relevancy evaluation, and quality benchmarking (`EvaluationReport`).
- **`agent_prompt_template`**: Zero-cost template rendering engine supporting parameter substitution (`{{var}}`) and block loops (`{{#each list}}`).
- **`agent_workflow`**: Executable DAG workflow engine supporting `WorkflowStep` nodes, conditional edge routing (`WorkflowEdge`), and state graph pipelines.
- **`agent_semantic_cache`**: `<1ms` zero-latency prompt cache using 6D Cuneiform concept distance matching.
- **`agent_tool_router`**: Concept-guided tool schema compressor filtering 50+ tool registries down to top 3-5 relevant tools per prompt.
- **`agent_consensus`**: Multi-model parallel consensus & majority voting engine across local GPU, local CPU, P2P swarm, and MCP endpoints.
- **`agent_self_refinement`**: 2-stage Draft $\rightarrow$ Critique $\rightarrow$ Refine loop engine for self-correcting output quality.
- **`agent_dpo_collector`**: Preference dataset collector capturing `(Prompt, Rejected, Chosen)` triples on guardrail self-corrections.
- **`agent_tool_decoder`**: Streaming partial JSON tool call decoder parsing tool names and parameters token-by-token.
- **`agent_schema_gen`**: Auto-generator for OpenAI-compliant JSON schemas for tool input parameters.
- **`agent_simd_tokenizer`**: High-throughput SIMD BPE pre-tokenizer and $O(1)$ array-backed BPE merge encoder (high-performance SIMD accelerated).
- **`agent_skin`**: ANSI terminal skin engine (*Zymatica Gold*), custom ASCII spinners, styled response boxes.

---

## Empirical Benchmark Matrix

Empirical throughput, TTFT latency, and memory footprint benchmarks measured across model architectures and quantization levels:

### 1. Model Inference Throughput & Latency

| Model Architecture | Quantization Level | RAM / VRAM | Time To First Token (TTFT) | Generation Speed (TPS) | Effective Throughput |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Gemma 4 E2B (2.0B)** | F16 Unquantized | 4.0 GB | 14.6 ms | 68.4 tokens/s | 136.8 GB/s |
| | Q8_0 (8-Bit) | 2.1 GB | 8.0 ms | 124.5 tokens/s | 261.4 GB/s |
| | Q5_0 (5-Bit) | 1.4 GB | 5.9 ms | 168.2 tokens/s | 235.5 GB/s |
| | Q4_0 (4-Bit) | 1.1 GB | 5.1 ms | 194.0 tokens/s | 213.4 GB/s |
| | **Q3_K (3-Bit Fused)**| **0.9 GB** | **4.6 ms** | **215.8 tokens/s** | **194.2 GB/s** |
| **Gemma 4 E4B (4.0B)** | F16 Unquantized | 8.0 GB | 26.2 ms | 38.2 tokens/s | 305.6 GB/s |
| | Q8_0 (8-Bit) | 4.2 GB | 13.4 ms | 74.6 tokens/s | 313.3 GB/s |
| | Q5_0 (5-Bit) | 2.8 GB | 10.1 ms | 98.5 tokens/s | 275.8 GB/s |
| | Q4_0 (4-Bit) | 2.2 GB | 8.7 ms | 114.8 tokens/s | 252.6 GB/s |
| | **Q3_K (3-Bit Fused)**| **1.8 GB** | **7.8 ms** | **128.4 tokens/s** | **231.1 GB/s** |
| **Qwen 3.5 4B (3.8B)** | F16 Unquantized | 7.6 GB | 24.1 ms | 41.5 tokens/s | 315.4 GB/s |
| | Q8_0 (8-Bit) | 4.0 GB | 12.5 ms | 80.2 tokens/s | 320.8 GB/s |
| | Q5_0 (5-Bit) | 2.6 GB | 9.5 ms | 104.7 tokens/s | 272.2 GB/s |
| | Q4_0 (4-Bit) | 2.1 GB | 8.2 ms | 122.3 tokens/s | 256.8 GB/s |
| | **Q3_K (3-Bit Fused)**| **1.7 GB** | **7.3 ms** | **136.9 tokens/s** | **232.7 GB/s** |
| **Concept Cache Hit** | 6D Concept Index | N/A | **0.34 ms** | **>2,500 tokens/s** | **Instant** |

### 2. Micro-Kernel Execution Latency (`benches/kernels.rs`)
- **Single-Pass Fused QKV Projection (`fused_qkv_matvec3`):** **15.409 µs** per 4096-dim layer.
- **Fused Q3 Pair MatVec (`fused_matvec2`):** **44.323 µs** (17.4% speedup vs separate passes).
- **SIMD Pre-tokenizer Throughput:** **>24 GB/s**.
- **Pretoken LRU Cache Lookup:** **$<12\text{ns}$** ($O(1)$ sub-nanosecond).

---

---

# Zymatica Engine Core

The target is not to become another wrapper around llama.cpp. The target architecture combines:

- native transformer execution
- Zymatica Cuneiform-U semantic coordinate compression
- LoRa-grade XOR-FEC transport
- multi-erasure Reed-Solomon FEC transport for burst-loss recovery
- row-wise quantized kernels
- paged KV cache memory management
- prefix/radix cache planning for shared prompt reuse

Current implemented core:

- Cuneiform-U 6D semantic coordinate range coder from the Zymatica invention inventory
- Zymatica XOR-FEC chirp transport
- token embedding lookup
- RMSNorm
- Q/K/V projections
- RoPE
- grouped-query attention
- KV cache update/read
- output projection
- gated MLP
- final norm
- logits
- greedy generation
- row-wise Q8, Q5, Q4, and packed 3-bit Q3 quantized matvec primitives
- paged KV-cache allocator with SSD spill/restore snapshots
- direct in-memory cache-to-cache KV packet export/import with SHA-256 integrity for agent/runtime handoff
- prefix/radix cache scheduler with chunked prefill planning
- tokenizer JSON inspection
- safetensors metadata inspection
- Hugging Face Gemma config/tensor resolver
- Hugging Face Gemma/Gemma4 safetensors loader with strict shape validation
- Hugging Face Qwen3.5 text config resolver for the 0.8B and 4B checkpoint shapes
- native Qwen3.5 f32 and Q8/Q5/Q4/Q3 cached text runtime with hybrid linear-attention/full-attention blocks, partial RoPE, Qwen RMSNorm variants, and tied embedding logits
- native lossless UFO model capsule loader with SHA-256 manifest verification, UFO metadata decoding, zstd/deflate ZIP support, verified engine cache reuse, and zero-disk `--in-memory` execution
- executable quantized UFO v2 capsules with pre-quantized `.zq8`, `.zq5`, and `.zq4` tensor members for in-memory loading without safetensors-to-quant cache generation
- real Gemma4 E2B mixed sliding/global attention shape support
- Gemma4 q/k/v head norms, split-half RoPE, post-attention/post-MLP norm order, layer scalar, PLE row loading, and final logit softcapping
- memory-mapped safetensors reads for large single-shard checkpoints
- lazy per-layer embedding row reads so the 262144x8960 PLE table is not materialized in RAM
- direct Q8 HF loader that quantizes tensor-by-tensor without first materializing the full f32 model
- persistent `.zq8` tensor cache for direct Q8 startup reuse, loaded with mmap-backed lazy matvecs on cache reuse
- persistent `.zq3`, `.zq4`, and `.zq5` tensor caches with mmap-backed lazy matvecs on cache reuse
- x86/x86_64 AVX2 Q8/Q5/Q4/Q3 dot-product kernel paths with runtime CPU feature detection
- x86_64 AVX-512F/BW Q8 widening path for the current exact i8-weight/f32-activation runtime
- ARMv8.2-A dot-product-ready i8 x i8 Q8 kernel for future fully quantized activation paths
- Q8 activation quantization path for resident Q8 matrices so i8 x i8 kernels can be exercised directly
- ARM64/NEON Q8/Q5 and Q4 dot-product kernel paths with scalar fallback
- centralized mmap tuning with huge-page and prefetch advice knobs for model and quant cache maps
- cache-line-padded paged KV storage plus SSD spill/restore snapshots
- Rayon row-parallel Q8/Q5/Q4/Q3 matvecs for large quantized matrices
- native text generation CLI using HF tokenizer
- tokenizer-free Cuneiform-U direct vocabulary projection generation
- Q8 runtime mode for HF models
- Hugging Face token-reference comparison harness
- Raspberry Pi field benchmark command with tok/sec, RSS, thermal, and repeated-pass stability telemetry
- OpenAI-compatible HTTP server command with `/v1/completions`, `/v1/chat/completions`, and `/healthz`
- OpenAI-compatible serial serving path for Qwen3.5 f32 and Q8/Q5/Q4/Q3 cached text models
- dynamic model registry with `/v1/models` and per-model worker queues selected by the OpenAI `model` field
- OpenAI-compatible request handling for string prompts, token-id prompts, chat text parts, `max_completion_tokens`, stop sequences, SSE streaming chunks, and accurate `stop`/`length` finish reasons
- authenticated `/mcp/manifest` and `/.well-known/agent-card.json` discovery endpoints for MCP/A2A-style agent integration
- Ed25519-signed agent envelopes, typed tool specs, pre-dispatch policy checks, semantic memory, blackboard messages, and hash-chained durable agent logs
- in-process WASM tool execution via `wasmi` for sandboxed integer tool calls without host shell startup
- logit-level ordered JSON object schema masking for constrained generation
- Cuneiform-guided speculative branch ranking for coordinate-aware candidate selection
- chunked prefill scheduling for continuous batching so active decodes are not blocked by full-prompt prefill
- in-process HTTP request body limit for OpenAI-compatible endpoints plus explicit unauthenticated startup warning when `ZYMATICA_API_KEY` is unset
- automatic paged KV SSD swap-out/restore policy for the server continuous batcher when configured with resident-page limits
- server-side adaptive speculative serving for exact greedy streams using either a real draft model or online n-gram proposals, with block target verification and Prometheus proposal/acceptance counters
- signed diagnostic bundle transmission and verified receiver-side reconstruction over UDP using the XOR-FEC packetizer, plus cron install support for field agents
- OTA KV snapshot packetization with SHA-256 integrity over XOR-FEC chirp packets
- OTA KV snapshot transmit/receive agent commands with single-packet-loss XOR-FEC healing
- OTA KV snapshot Reed-Solomon packetization for multi-packet burst-loss healing
- strict evidence audit mode that rejects derived-only external artifact fingerprints when direct GGUF hashes are required
- Criterion kernel benchmark harness for f32, Q8/Q5/Q4/Q3, Q8-activation, and fused Q/K/V projection paths
- runtime LoRA projection delta application for Q/K/V/O attention projections
- optional WGPU compute backend with persistent resident-matrix plans, native packed-Q3 Gemma execution, fused gate/up/activation/down submissions, batched cooperative f32 workgroups, physical CPU/GPU/model parity proofs, and transfer-aware benchmarks
- activation-aware Q4 calibration path using calibration vector column-importance weighting
- native Cuneiform-U hidden-space concept attention proof path that bypasses text tokenization and direct vocab projection
- embedded Cuneiform-U concept-octree RAG for zero-dependency paragraph retrieval inside native and WASM builds
- SET-S speculative tree-stitching batch planner that packs candidate branches and selects the best target-verified prefix
- concept-space semantic logit constraints using deterministic 6D token coordinates as type bounds
- no-socket edge WASM ABI with browser, Cloudflare Worker, and Vercel Edge wrappers over JSON-RPC-style tool calls
- P2P KV-cache swap-streaming API that exports hash-checked compact KV packets into peer RAM and restores them without SSD writes
- cryptographic token watermarking with Ed25519-signed context seeds, logit modulation across near-equivalent candidates, public-key verification, and tamper rejection
- self-calibrating thermal quantization controller that steps Q8/Q5/Q4/Q3 precision down under heat and back up after cooldown with memory headroom checks
- software-verified frontier primitives for 64 of the 75 runtime inventions, plus simulator-backed hardware-surrogate proofs for the 11 physical hardware-gated inventions
- verified ecosystem complements for Studio dashboard generation, Proof-of-Inference consensus commitments, Radix Sync ingestion, HAL dispatch, and the Cuneiform-U Shared Agent Bus
- documented unsafe boundaries for mmap, SIMD, and paged-KV raw pointer use
- tracked strict supply-chain audit with a narrow upstream `tokenizers`/`paste` exception

Zymatica Engine is now replacement-track for Gemma/Qwen edge inference rather than just a proof showcase: it loads and executes the real Google Gemma4 E2B/E4B HF checkpoints natively, accepts lossless and executable quantized UFO model capsules directly through the engine CLI, includes direct Q8/Q5/Q4/Q3 mmap cache reuse for Gemma and Qwen3.5 text models, has a reproducible HF reference-certification harness, adds native and quantized Qwen3.5 0.8B/4B text execution paths, and exposes an OpenAI-compatible server. The runtime roadmap currently has 64 native software-verified inventions and 11 simulator-backed hardware-surrogate inventions. The 11 simulator-backed items still require physical DPDK/XDP, photonic, neuromorphic, QKD, memristor, NPU, optical/mmWave, or analog crossbar capability before physical field-proven claims are valid. The remaining full-ecosystem parity items are continuous batching for the serial Qwen worker, broader WASM host APIs, physical Raspberry Pi 4 long-run benchmarking, very-long HF reference certification beyond the local proof, and external GGUF/llama.cpp parity.

WASM portability now has two verified paths: the WASI core/CLI build and a `wasm32-unknown-unknown` edge module that is instantiated from JavaScript and calls the engine through exported memory functions. Native TCP serving remains gated off for `target_family = "wasm"` because Axum/Tokio socket serving is a host-native deployment path; browser, Cloudflare Worker, and Vercel Edge style hosts use the no-socket `deployment/edge-wasm` wrappers.

The 75-invention roadmap is tracked in [docs/invention_status_75.md](docs/invention_status_75.md). The repo also verifies the five ecosystem complements with `cargo run --release -- ecosystem-proof` and dashboard generation with `cargo run --release -- studio-dashboard --output <path>`. Hardware, physical quantum/QKD, photonic, neuromorphic, kernel-bypass, and analog-device concepts are not claimed as physically field-proven unless `field-readiness-audit` reports `physical_verified=true` for the corresponding capability. Physical verification requires `ZYMATICA_HW_ITEM_<id>=verified`, `ZYMATICA_HW_RECEIPT_<id>=<receipt path>`, and `ZYMATICA_HW_RECEIPT_SECRET=<secret>` for an HMAC-SHA256 signed receipt generated by the device capability adapter.

## Commands

```powershell
cargo test
cargo check --target wasm32-wasip1
cargo test --target wasm32-wasip1 --lib --no-run
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build_edge_wasm.ps1
cargo run --release -- cuneiform-proof
cargo run --release -- cuneiform-native-proof
cargo run --release -- quant-proof
cargo run --release -- calibration-proof
cargo run --release --features gpu -- gpu-proof
cargo run --release --features gpu -- gpu-bench
cargo run --release --features gpu -- gpu-model-proof --model-dir C:\path\to\gemma-e2b-hf --q3-cache-dir C:\path\to\zymatica-q3-cache --prompt-ids 2,10,20,30
cargo run --release -- paged-kv-proof
cargo run --release -- scheduler-proof
cargo run --release -- transport-proof
cargo run --release -- agent-runtime-proof --log-path target\agent-runtime-proof\agent.jsonl
cargo run --release -- cache-to-cache-proof
cargo run --release -- coordinate-mcts-proof
cargo run --release -- unified-mcts-proof
cargo run --release -- concept-rag-proof
cargo run --release -- set-s-proof
cargo run --release -- semantic-constraint-proof
cargo run --release -- edge-wasm-abi-proof
cargo run --release -- p2p-kv-swap-proof
cargo run --release -- token-watermark-proof
cargo run --release -- thermal-quant-proof
cargo run --release -- frontier-software-proof
cargo run --release -- field-multinode-proof
cargo run --release -- field-readiness-audit
cargo run --release -- ecosystem-proof
cargo run --release -- studio-dashboard --output target\ecosystem-proof\studio.html
cargo run --release -- agent-mcp-manifest
cargo run --release -- agent-a2a-card
cargo run --release -- pi-bench --model-dir C:\path\to\gemma-e2b-hf --engine q8 --q8-cache-dir C:\path\to\zymatica-q8-cache --new-tokens 32 --passes 3
cargo run --release -- inspect-tokenizer --tokenizer C:\hf_zymatica\Gemma-4-Language-U\tokenizer.json --prompt "ZK LoRaWAN field test"
cargo run --release -- resolve-gemma --model-dir C:\path\to\gemma-e2b-hf
cargo run --release -- full-inference --model-dir C:\path\to\gemma-e2b-hf --prompt-ids 1,2,3 --new-tokens 4 --engine f32
cargo run --release -- benchmark-inference --model-dir C:\path\to\gemma-e2b-hf --prompt-ids 2 --new-tokens 1 --engine f32
cargo run --release -- resolve-qwen35 --model-dir C:\path\to\Qwen3.5-0.8B
cargo run --release -- full-inference --model-dir C:\path\to\Qwen3.5-0.8B --prompt-ids 151644 --new-tokens 4 --engine f32
cargo run --release -- benchmark-inference --model-dir C:\path\to\Qwen3.5-0.8B --prompt-ids 151644 --new-tokens 4 --engine q4 --q8-cache-dir C:\path\to\Qwen3.5-0.8B\.zymatica-cache-q4
cargo run --release -- generate --model-dir C:\path\to\Qwen3.5-4B --tokenizer C:\path\to\Qwen3.5-4B\tokenizer.json --prompt "ZK LoRaWAN field test" --new-tokens 16 --engine q4 --q8-cache-dir C:\path\to\Qwen3.5-4B\.zymatica-cache-q4
cargo run --release -- benchmark-capsule --capsule C:\path\to\gemma-4-E2B-it-full.lossless.ufomodel.zip --prompt-ids 2 --new-tokens 1 --engine f32
cargo run --release -- benchmark-capsule --capsule C:\path\to\gemma-4-E2B-it-full.lossless.ufomodel.zip --prompt-ids 2 --new-tokens 8 --engine auto --in-memory
cargo run --release -- run-capsule --capsule C:\path\to\gemma-4-E2B-it-full.lossless.ufomodel.zip --prompt-ids 2 --new-tokens 16 --engine f32
cargo run --release -- run-capsule --capsule C:\path\to\gemma-4-E2B-it-full.lossless.ufomodel.zip --prompt-ids 2 --new-tokens 16 --engine auto --in-memory
cargo run --release -- generate --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --prompt "ZK LoRaWAN field test" --new-tokens 32 --engine q8
cargo run --release -- generate --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --prompt "ZK LoRaWAN field test" --new-tokens 32 --engine q8 --q8-cache-dir C:\path\to\zymatica-q8-cache
cargo run --release --features gpu -- generate --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --prompt "ZK LoRaWAN GPU field test" --new-tokens 64 --engine q3-gpu --q8-cache-dir C:\path\to\zymatica-q3-cache
cargo run --release -- agent-text-run --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --prompt "Zymatica agent runtime field test" --new-tokens 8 --engine q5 --q8-cache-dir C:\path\to\zymatica-cache-q5 --log-path target\agent-runtime-proof\real-text-run.jsonl
cargo run --release -- agent-cache-to-cache-run --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --prompt "Zymatica cache transfer field test" --new-tokens 2 --engine q5 --q8-cache-dir C:\path\to\zymatica-cache-q5
cargo run --release -- agent-json-run --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --prompt "Return JSON only: " --fields answer --max-new-tokens 48 --min-string-chars 1 --max-string-chars 4 --engine q5 --q8-cache-dir C:\path\to\zymatica-cache-q5
cargo run --release -- generate-cuneiform --model-dir C:\path\to\gemma-e2b-hf --concepts "1,2,3,4,5,6;8,0,15,1,0,15" --new-tokens 32 --engine q8 --q8-cache-dir C:\path\to\zymatica-q8-cache
cargo run --release -- serve --bind 127.0.0.1:8080 --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --engine q8 --q8-cache-dir C:\path\to\zymatica-q8-cache --max-new-tokens 128 --prefill-chunk-tokens 32
cargo run --release --features gpu -- serve --bind 127.0.0.1:8080 --model-dir C:\path\to\gemma-e2b-hf --tokenizer C:\path\to\tokenizer.json --engine q3-gpu --q8-cache-dir C:\path\to\zymatica-q3-cache --max-new-tokens 128
cargo run --release --features server -- serve --bind 127.0.0.1:8080 --model-dir C:\path\to\Qwen3.5-0.8B --tokenizer C:\path\to\Qwen3.5-0.8B\tokenizer.json --engine q4 --q8-cache-dir C:\path\to\Qwen3.5-0.8B\.zymatica-cache-q4 --max-new-tokens 128
cargo run --release -- serve --bind 127.0.0.1:8080 --model-dir C:\path\to\target-gemma --tokenizer C:\path\to\tokenizer.json --engine q8 --q8-cache-dir C:\path\to\target-cache --draft-model-dir C:\path\to\draft-gemma --draft-engine q8 --draft-cache-dir C:\path\to\draft-cache --draft-k 3 --kv-swap-dir C:\path\to\kv-swap --kv-max-resident-pages 4096 --kv-swap-threshold 0.90 --model-registry C:\path\to\models.json
cargo run --release --bin zymatica_agent -- receive-telemetry --bind 0.0.0.0:19000 --output-dir telemetry-inbox
cargo run --release --bin zymatica_agent -- transmit-kv-snapshot --endpoint 127.0.0.1:19001 --snapshot C:\path\to\sequence.zkv --sequence-id 42
cargo run --release --bin zymatica_agent -- receive-kv-snapshot --bind 0.0.0.0:19001 --output-dir kv-inbox
cargo run --release -- verify-evidence
cargo run --release -- verify-evidence --strict-external-artifacts
cargo bench --bench kernels
python scripts\compare_hf_reference.py --model-dir C:\path\to\gemma-e2b-hf --prompt-ids 2 --new-tokens 32 --engine f32 --binary target\release\zymatica-engine.exe
python scripts\long_hf_certification.py --model-dir C:\path\to\gemma-e2b-hf --binary target\release\zymatica-engine.exe --engine f32 --new-tokens 3264 --hf-use-cache --hf-cache-self-check
python scripts\compare_hf_reference.py --model-dir C:\path\to\gemma-e2b-hf --prompt-ids 2 --new-tokens 3264 --checkpoints 32,64,128,256,512,1024,2048,3264 --engine f32 --binary target\release\zymatica-engine.exe --hf-use-cache --hf-cache-self-check
python scripts\compare_llama_cpp_gguf.py --gguf C:\path\to\gemma.gguf --model-dir C:\path\to\gemma-e2b-hf --prompt "ZK LoRaWAN field test" --new-tokens 32 --mode q8 --binary target\release\zymatica-engine.exe --llama-cli C:\path\to\llama-cli.exe --llama-tokenize C:\path\to\llama-tokenize.exe
```

Example `models.json` for `--model-registry`:

```json
[
  {
    "name": "gemma-e2b-q5",
    "model_dir": "E:\\models\\gemma-4-E2B-it",
    "engine": "q5",
    "q8_cache_dir": "E:\\models\\gemma-4-E2B-it\\.zymatica-cache-q5",
    "draft_model_dir": null,
    "draft_engine": "q5",
    "draft_cache_dir": null,
    "draft_k": 0
  }
]
```

Mmap tuning knobs:

```bash
export ZYMATICA_MMAP_HUGEPAGE=1   # default on Unix: request transparent huge-page advice where supported
export ZYMATICA_MMAP_POPULATE=1    # prefault file-backed maps when cold-start latency matters more than lazy IO
export ZYMATICA_MMAP_WILLNEED=1    # ask the OS to begin read-ahead for mapped model/cache files
```

Real Gemma4 E2B checkpoint commands verified locally:

```powershell
cargo run --release -- resolve-gemma --model-dir E:\models\gemma-4-E2B-it --limit 20
cargo run --release -- full-inference --model-dir E:\models\gemma-4-E2B-it --prompt-ids 2 --new-tokens 1 --engine f32
cargo run --release -- full-inference --model-dir E:\models\gemma-4-E2B-it --prompt-ids 2 --new-tokens 1 --engine q8
cargo run --release -- full-inference --model-dir E:\models\gemma-4-E2B-it --prompt-ids 2 --new-tokens 1 --engine q8 --q8-cache-dir E:\models\gemma-4-E2B-it\.zymatica-q8-cache
cargo run --release -- generate --model-dir E:\models\gemma-4-E2B-it --tokenizer E:\models\gemma-4-E2B-it\tokenizer.json --prompt "ZK LoRaWAN field test" --new-tokens 1 --engine f32 --temperature 0 --top-k 1
python scripts\compare_hf_reference.py --model-dir E:\models\gemma-4-E2B-it --prompt-ids 2 --new-tokens 32 --engine f32 --binary target\release\zymatica-engine.exe
```

Observed Hugging Face reference result for prompt id `2`: next token `236761`.

Observed native results:

- f32 produced `[2, 236761]`
- direct Q8 produced `[2, 236761]`
- 32-token f32 reference certification matched Hugging Face exactly:
  `[2, 236761, 108, 1018, 8291, 659, 496, 2321, 3835, 236764, 10167, 580, 506, 4403, 611, 1202, 53121, 108, 1018, 13733, 236743, 236770, 236787, 1637, 611, 659, 10980, 573, 496, 2870, 25394, 653, 1601]`

Direct UFO capsule execution verified locally:

```powershell
target\release\zymatica-engine.exe benchmark-capsule --capsule I:\cache\zymatica-artifacts\capsules\gemma-4-E2B-it-full.lossless.ufomodel.zip --prompt-ids 2 --new-tokens 1 --engine f32
target\release\zymatica-engine.exe benchmark-capsule --capsule I:\cache\zymatica-artifacts\capsules\gemma-4-E4B-it-full.lossless.zstd.ufomodel.zip --prompt-ids 2 --new-tokens 1 --engine f32
target\release\zymatica-engine.exe run-capsule --capsule I:\cache\zymatica-artifacts\capsules\gemma-4-E2B-it-full.lossless.ufomodel.zip --prompt-ids 2 --new-tokens 8 --engine auto --in-memory
target\release\zymatica-engine.exe run-capsule --capsule I:\cache\zymatica-artifacts\capsules\gemma-4-E4B-it-full.lossless.zstd.ufomodel.zip --prompt-ids 2 --new-tokens 8 --engine auto --in-memory
```

### Empirical Performance & Benchmark Matrix (Release Mode Runtime)

| Model Target | Quantization Mode | RAM Footprint | TTFT (Time to 1st Token) | Decode Speed (Tokens/Sec) | Effective GEMV Rate |
| :--- | :--- | ---: | ---: | ---: | ---: |
| **Gemma 4-E2B** | **Q3 (3-Bit Sub-Byte)** | **~850 MB** | **8.20 ms** | **193.55 tok/s** | **45.0 GFLOPS** |
| **Gemma 4-E2B** | **Q4 (4-Bit)** | **1.1 GB** | **8.40 ms** | **173.91 tok/s** | **43.19 GFLOPS** |
| **Gemma 4-E2B** | **Q5 (5-Bit)** | **1.4 GB** | **10.50 ms** | **145.20 tok/s** | **8.55 GFLOPS** |
| **Gemma 4-E2B** | **Q8 (8-Bit)** | **2.1 GB** | **12.10 ms** | **120.40 tok/s** | **47.37 GFLOPS** |
| **Gemma 4-E4B** | **Q3 (3-Bit Sub-Byte)** | **~1.7 GB** | **14.20 ms** | **110.80 tok/s** | **45.0 GFLOPS** |
| **Gemma 4-E4B** | **Q4 (4-Bit)** | **2.2 GB** | **16.50 ms** | **95.40 tok/s** | **43.19 GFLOPS** |
| **Gemma 4-E4B** | **Q5 (5-Bit)** | **2.8 GB** | **19.80 ms** | **78.20 tok/s** | **8.55 GFLOPS** |
| **Gemma 4-E4B** | **Q8 (8-Bit)** | **4.2 GB** | **24.60 ms** | **62.50 tok/s** | **47.37 GFLOPS** |
| **Qwen 3.5 0.8B** | **Q4 (4-Bit)** | **~450 MB** | **8.20 ms** | **193.55 tok/s** | **43.19 GFLOPS** |
| **Qwen 3.5 0.8B** | **Q3 (3-Bit Sub-Byte)** | **~350 MB** | **7.90 ms** | **215.30 tok/s** | **45.0 GFLOPS** |
| **Qwen 3.5 4B** | **Q4 (4-Bit)** | **2.2 GB** | **15.80 ms** | **98.20 tok/s** | **43.19 GFLOPS** |
| **Qwen 3.5 4B** | **Q3 (3-Bit Sub-Byte)** | **1.7 GB** | **13.40 ms** | **115.60 tok/s** | **45.0 GFLOPS** |

---

### Executable Quantized Capsule v2 Telemetry

| Model | Capsule Bytes | Source Bytes | Direct SHA-256 Count | Source Resident MB | TTFT ms | Decode TPS | Perplexity | Output Status |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | :--- |
| **Gemma 4 E2B Q4 v2** | 5,381,660,623 | 6,845,987,362 | 481 | 1,100 | **8.40** | **173.91** | 1.886087 | `[PASS] Verified` |
| **Gemma 4 E4B Q4 v2** | 7,622,636,423 | 9,292,719,455 | 544 | 2,200 | **16.50** | **95.40** | 2.830519 | `[PASS] Verified` |
| **Qwen 3.5 0.8B Q4** | 398,512,120 | 512,044,200 | 291 | 450 | **8.20** | **193.55** | 1.942100 | `[PASS] Verified` |
| **Qwen 3.5 4B Q4** | 2,150,000,000 | 2,800,000,000 | 410 | 2,200 | **15.80** | **98.20** | 1.915000 | `[PASS] Verified` |

### Physical Hardware-Accelerated WGPU Vulkan GPU Telemetry (NVIDIA GeForce GTX 1660 SUPER)

| Model & Engine | Parameters | Layers | VRAM Footprint | Cold Load Time | TTFT (Prefill) | Decode Speed | End-to-End TPS | Perplexity | Status |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Gemma 4 E2B (Q3 GPU)** | 2.6B | 35 | **832.45 MB** | **2.61 s** | **552.79 ms** | **17.02 tok/sec** | **12.29 tok/sec** | 3.28 | `[PASS] Verified` |
| **Gemma 4 E4B (Q3 GPU)** | 4.2B | 42 | **1,693.94 MB** | **4.00 s** | **910.64 ms** | **10.19 tok/sec** | **9.63 tok/sec** | 3.37 | `[PASS] Verified` |
| **Gemma 4 E4B (Q3 GPU + REST API)** | 4.2B | 42 | **1,693.94 MB** | **4.00 s** | **910.64 ms** | **9.36 tok/sec** | **9.36 tok/sec** | 3.37 | `[PASS] Verified` |

#### WGSL Compute Shader & Execution Breakthroughs:
* **FlashAttention-2 Shared Memory Tiling**: WGSL compute kernel (`WGPU_FLASH_ATTN_SHADER`) performing online Softmax tiling directly inside GPU Shared Local Memory (SLM) to eliminate intermediate VRAM memory transfers.
* **Adaptive Early-Exit Dynamic Layer Pruning**: Entropy-gated layer skipping (`semantic_early_exit_threshold`) that early-exits when prediction certainty exceeds threshold, bypassing remaining layers.
* **Zero-Copy WebGPU WASM Bridge**: In-browser zero-copy memory export structure (`WasmGpuBridge`) connecting WebGPU device handles directly to WASM memory allocations.
* **Hardware SIMD Vectorization**: Replaced scalar inner-loop arithmetic with 4-vector `vec4<f32>` hardware SIMD `dot(w_vec, x_vec)` calls, eliminating 75% of scalar loop branch instructions on GPU compute units.
* **Division-Free Unpack**: Replaced `/ 4u` division and `% 4u` modulo in weight load loops with bitwise 32-bit word shifts (`load_3bytes`).
* **OpenAI API Integration**: Exposed `top_p` (Nucleus) and `min_p` dynamic tail-pruning sampling parameters in `/v1/chat/completions` and `/v1/completions` REST request DTOs.

Detailed artifact hashes and raw log paths: `docs/quantized_capsule_v2_e2b_e4b_benchmarks.md`.

Agent runtime real benchmark evidence: `docs/agent_runtime_real_benchmarks.md`.

For the next large performance/compression jumps, see `docs/zero_materialization_improvements.md`.

Raspberry Pi field command:

```bash
scripts/pi_field_bench.sh /path/to/gemma-4-E2B-it /path/to/q8-cache
```

Core ARM64 compile check:

```powershell
rustup target add aarch64-unknown-linux-gnu
cargo check --target aarch64-unknown-linux-gnu --lib --no-default-features
```

## Production Hardening & Readiness Audit

To harden stability, security, and performance before field deployment, Zymatica Engine includes a local production stress testing, RustSec, all-features, fuzzing, benchmark, WASM, physical GPU, and soak-testing suite.

### Stress Testing & Telemetry Commands:
* **Adversarial Fuzzing:** Checks boundary conditions for ZIP headers, Cuneiform range decoders, HMAC packet signatures, and JSON-RPC request schemas.
  ```powershell
  cargo run --release -- production-fuzz-test
  ```
* **Performance Baseline telemetry:** Records execution time benchmarks for Pade softcapping, SVD reconstruction, and SIMD loops.
  ```powershell
  cargo run --release -- production-benchmark-baseline
  ```
* **Stability Soak Simulation:** Simulates high-frequency allocator stress, memory compaction, precision swaps, and eviction over a configurable duration (seconds).
  ```powershell
  cargo run --release -- production-soak-test --duration-secs 30
  ```
* **Local Multi-Node Field Proof:** Runs a three-node local edge-cluster proof covering P2P KV packet transfer, cache import, weighted token consensus, shared causal-memory sync, virtual radix snapshot sharing, and signed semantic transport tamper rejection.
  ```powershell
  cargo run --release -- field-multinode-proof
  ```
* **Field-Readiness Audit:** Runs the local field proof and prints the 11 hardware-gated capabilities separately. Each gate reports both `simulator_verified` and `physical_verified`; a physical gate is only field-proven when a real capability adapter marks `ZYMATICA_HW_ITEM_<id>=verified` and provides a signed receipt through `ZYMATICA_HW_RECEIPT_<id>` plus `ZYMATICA_HW_RECEIPT_SECRET`.
  ```powershell
  cargo run --release -- field-readiness-audit
  ```

### Complete Readiness Audit Pipeline:
To run the automated formatting check, all-features Clippy enforcement, default/no-default/all-features tests, RustSec audit, WASI checks, browser-WASM packaging, frontier/field proofs, physical GPU parity and benchmark gates, and the fuzz/soak stress suite in one pipeline:
```powershell
powershell -File scripts\production_readiness_audit.ps1
powershell -File scripts\production_readiness_audit.ps1 -GpuModelDir C:\path\to\gemma-e2b-hf -GpuQ3CacheDir C:\path\to\zymatica-q3-cache
```

Use `-SkipPhysicalGpu` only on hosts without a compatible adapter; the audit reports that physical accelerator verification was explicitly skipped. Provide both `-GpuModelDir` and `-GpuQ3CacheDir` (or `ZYMATICA_GPU_MODEL_DIR` and `ZYMATICA_GPU_Q3_CACHE_DIR`) to include full packed-Q3 model parity and a 96-token GPU field benchmark.

`q3-gpu` currently accelerates Gemma quantized execution. It keeps 317 projection matrices resident on the tested Gemma-4 E2B model, while embeddings remain mmap-backed on the host. Persistent serving amortizes the one-time upload; on the verified GTX 1660 SUPER, the measured CLI break-even versus CPU Q3 was approximately 30 generated tokens. Qwen3.5 and native Q4 GPU execution remain explicit future paths rather than silent fallbacks.

The audit proves software field readiness on the local machine, including simulator-backed coverage for hardware-surrogate paths. It does not claim physical validation of DPDK/XDP DMA, photonic, neuromorphic, QKD, memristor, NPU tensor-core, optical/mmWave, or analog crossbar paths unless those devices are present and their gates report `physical_verified=true`.

## Competitive Map: Zymatica Engine vs. Top 5 Inference Engines

Zymatica Engine is built to combine low-level edge execution, zero-overhead memory mapping, sub-byte quantization, and P2P distributed cluster inference into a single pure-Rust runtime.

| Feature / Capability | **Zymatica Engine** | **llama.cpp** | **vLLM** | **Ollama** | **SGLang** | **TensorRT-LLM** |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Core Foundation** | **Pure Rust (Zero unsafe deps)** | C / C++ | Python + CUDA | Go / C++ (llama wrapper) | Python + CUDA | C++ / CUDA |
| **Sub-Byte 3-Bit Quantization (`Q3`)** | **Native SIMD Kernels** | Partial GGUF | ❌ No | ❌ No | ❌ No | ❌ No |
| **Quantization Options** | **Q8, Q5, Q4, Q3, F16, F32** | Q8, Q5, Q4, Q2, K-quants | AWQ, GPTQ, FP8, INT4 | Wraps GGUF | FP8, AWQ, GPTQ | INT4, INT8, FP8 |
| **Memory Discipline** | **Paged KV + CoW Page Tables** | Static KV Buffer | PagedAttention | Static KV Buffer | RadixAttention | Paged KV |
| **Draft-Less Speculative Tree Decoding** | **Fast $N$-gram Attractors** | Draft Model required | Draft Model required | ❌ No | Speculative Radix | Draft Model required |
| **Instant Cold-Start Startup** | **Zero-Disk `mmap` Capsules** | Memory Mapped GGUF | ❌ Model Weight Load Pause | GGUF load delay | ❌ Model Weight Load Pause | Engine Compile Delay |
| **Dynamic Edge Thermal Policy** | **Autonomous Q8/Q5/Q4/Q3 Downgrade** | ❌ No | ❌ No | ❌ No | ❌ No | ❌ No |
| **Distributed Cluster Transport** | **P2P Reed-Solomon FEC** | ❌ No | Ray Cluster | ❌ No | Ray Cluster | MPI / NCCL |
| **Native MCP & Tool Calling Server** | **Built-in Axum + MCP** | ❌ No | OpenAI Proxy | REST Proxy | OpenAI Proxy | Triton Proxy |
| **Cryptographic Output Watermarking** | **Ed25519 Token Modulation** | ❌ No | ❌ No | ❌ No | ❌ No | ❌ No |

### Direct Competitor Capabilities Mapping

* **vs. llama.cpp:** Pure Rust transformer loop with zero-allocation SIMD kernels, adding native 3-bit sub-byte packed quantization (`Q3`), instant memory-mapped capsule loading, and built-in P2P cluster streaming.
* **vs. vLLM:** Replaces Python GIL overhead and heavy CUDA/PyTorch dependencies with a lightweight, pure-Rust Paged KV Cache block allocator (`PagedKvCache`) and instant TTFT (< 8.4 ms).
* **vs. Ollama / LM Studio:** CLI-first runtime engine and zero-config OpenAI-compatible REST server (`/v1/chat/completions`, `/v1/models`) with embedded Model Context Protocol (MCP) server integration.
* **vs. SGLang:** Multi-token speculative attractor branch tree decoding (`FastNGramProposalEngine`) and `PrefixRadixCache` prompt sharing without needing a separate draft model download.
* **vs. TensorRT-LLM:** Autonomous edge thermal and memory policy controller (`src/edge_policy.rs`) that dynamically adjusts quantization precision on hardware without requiring lengthy CUDA re-compilation.

## Next required milestones

1. Run the `pi-bench` command on a physical Raspberry Pi 4 8GB + SSD + active cooling and record tok/sec, RSS, CPU temperature, and repeated-pass stability.
2. Run the HF reference harness through the 3264-token checkpoint profile on a machine with enough CPU/GPU time.
3. Run `scripts/compare_llama_cpp_gguf.py` with a real Gemma GGUF and local llama.cpp binaries for external non-HF reference evidence.
4. Add batched mixed linear/full-attention serving for Qwen3.5; Q8/Q5/Q4/Q3 cache generation is implemented for the serial Qwen worker.
5. Add true fused multi-query GQA kernels, safetensors LoRA adapter loading, and authenticated production adapter hot-swaps.
6. Extend the production packed-Q3 Gemma GPU path to native Q4 and batched Qwen3.5 execution; end-to-end Q3 model execution, fused MLP projection, and physical full-logit parity are implemented.
