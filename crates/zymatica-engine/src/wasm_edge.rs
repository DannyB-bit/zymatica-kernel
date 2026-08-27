use crate::concept_constraints::{ConceptBounds6D, ConceptConstraintMask};
use crate::concept_rag::{ConceptRagIndex, project_text_to_concept};
use crate::cuneiform::{Concept6D, token_id_to_concept};
use crate::speculative::{
    CoordinateBranch, TreeStitchConfig, stitch_speculative_tree, verify_stitched_tree_batch,
};
use serde_json::{Value, json};

pub fn handle_edge_json(input: &str) -> String {
    let payload: Value = match serde_json::from_str(input) {
        Ok(value) => value,
        Err(err) => {
            return json_rpc_error(Value::Null, -32700, format!("Parse error: {err}")).to_string();
        }
    };
    let id = payload.get("id").cloned().unwrap_or(Value::Null);
    let method = payload
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let response = match method {
        "tools/list" => Ok(json!({
            "tools": [
                {
                    "name": "concept_project",
                    "description": "Project text directly into a deterministic 6D Cuneiform-U concept coordinate.",
                    "inputSchema": {"type": "object", "required": ["text"]}
                },
                {
                    "name": "concept_rag",
                    "description": "Retrieve nearest document paragraphs from an embedded Cuneiform-U concept octree.",
                    "inputSchema": {"type": "object", "required": ["query", "documents"]}
                },
                {
                    "name": "concept_mask_count",
                    "description": "Count vocabulary entries allowed by a 6D semantic type bounding box.",
                    "inputSchema": {"type": "object", "required": ["vocab_size", "min", "max"]}
                },
                {
                    "name": "set_s_select",
                    "description": "Pack speculative branch candidates and select the best target-verified prefix.",
                    "inputSchema": {"type": "object", "required": ["branches", "target_tokens"]}
                }
            ]
        })),
        "tools/call" => {
            let params = payload.get("params").unwrap_or(&Value::Null);
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let arguments = params.get("arguments").unwrap_or(&Value::Null);
            call_edge_tool(name, arguments)
        }
        other => Err(format!("Method Not Found: '{other}'")),
    };
    match response {
        Ok(result) => json!({"jsonrpc": "2.0", "result": result, "id": id}).to_string(),
        Err(message) => json_rpc_error(id, -32602, message).to_string(),
    }
}

fn call_edge_tool(name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "concept_project" => {
            let text = required_str(arguments, "text")?;
            Ok(concept_json(project_text_to_concept(text)))
        }
        "concept_rag" => {
            let query = required_str(arguments, "query")?;
            let documents = arguments
                .get("documents")
                .and_then(Value::as_array)
                .ok_or_else(|| "concept_rag requires documents array".to_string())?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "concept_rag documents must be strings".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .max(1) as usize;
            let index = ConceptRagIndex::from_paragraphs(&documents);
            let hits = index
                .query(query, limit)
                .into_iter()
                .map(|hit| {
                    json!({
                        "id": hit.id,
                        "text": hit.text,
                        "distance": hit.distance,
                        "concept": concept_json(hit.concept),
                    })
                })
                .collect::<Vec<_>>();
            Ok(json!({
                "query_concept": concept_json(project_text_to_concept(query)),
                "tree_nodes": index.tree_node_count(),
                "hits": hits,
            }))
        }
        "concept_mask_count" => {
            let vocab_size = arguments
                .get("vocab_size")
                .and_then(Value::as_u64)
                .ok_or_else(|| "concept_mask_count requires vocab_size".to_string())?
                as usize;
            let min = parse_concept(arguments.get("min"))?;
            let max = parse_concept(arguments.get("max"))?;
            let mask = ConceptConstraintMask::single(
                ConceptBounds6D::new(min, max).map_err(|err| err.to_string())?,
            );
            let allowed = (0..vocab_size)
                .filter(|token_id| mask.allows_token(*token_id))
                .count();
            Ok(json!({"allowed": allowed, "vocab_size": vocab_size}))
        }
        "set_s_select" => {
            let target_tokens = arguments
                .get("target_tokens")
                .and_then(Value::as_array)
                .ok_or_else(|| "set_s_select requires target_tokens array".to_string())?
                .iter()
                .map(|value| {
                    value
                        .as_u64()
                        .map(|token| token as usize)
                        .ok_or_else(|| "target_tokens values must be unsigned integers".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let branches = parse_branches(arguments)?;
            let target_concept = arguments
                .get("target_concept")
                .map(|value| parse_concept(Some(value)))
                .transpose()?
                .unwrap_or_else(|| {
                    target_tokens
                        .last()
                        .copied()
                        .map(token_id_to_concept)
                        .unwrap_or_else(|| Concept6D::new(0, 0, 0, 0, 0, 0))
                });
            let config = TreeStitchConfig::default();
            let batch = stitch_speculative_tree(&branches, &[], target_concept, config);
            let selected = verify_stitched_tree_batch(&batch, &target_tokens, config)
                .ok_or_else(|| "set_s_select received no branches".to_string())?;
            Ok(json!({
                "packed_tokens": batch.packed_tokens,
                "offsets": batch.offsets,
                "selected_branch": selected.branch_id,
                "accepted_prefix_len": selected.accepted_prefix_len,
                "selected_tokens": selected.tokens,
                "score": selected.score,
            }))
        }
        other => Err(format!("unknown edge tool '{other}'")),
    }
}

fn parse_branches(arguments: &Value) -> Result<Vec<CoordinateBranch>, String> {
    arguments
        .get("branches")
        .and_then(Value::as_array)
        .ok_or_else(|| "set_s_select requires branches array".to_string())?
        .iter()
        .map(|branch| {
            let tokens = branch
                .get("tokens")
                .and_then(Value::as_array)
                .ok_or_else(|| "each branch requires tokens array".to_string())?
                .iter()
                .map(|value| {
                    value
                        .as_u64()
                        .map(|token| token as usize)
                        .ok_or_else(|| "branch token values must be unsigned integers".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let concepts = if let Some(values) = branch.get("concepts").and_then(Value::as_array) {
                values
                    .iter()
                    .map(|value| parse_concept(Some(value)))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                tokens.iter().copied().map(token_id_to_concept).collect()
            };
            let logprob = branch.get("logprob").and_then(Value::as_f64).unwrap_or(0.0) as f32;
            Ok(CoordinateBranch {
                tokens,
                concepts,
                logprob,
                score: 0.0,
            })
        })
        .collect()
}

fn parse_concept(value: Option<&Value>) -> Result<Concept6D, String> {
    let value = value.ok_or_else(|| "concept value is missing".to_string())?;
    let axes = value
        .as_array()
        .ok_or_else(|| "concept must be a six-value array".to_string())?;
    if axes.len() != 6 {
        return Err("concept must contain exactly six axes".to_string());
    }
    let mut out = [0_u8; 6];
    for (idx, axis) in axes.iter().enumerate() {
        let value = axis
            .as_u64()
            .ok_or_else(|| format!("concept axis {idx} must be an unsigned integer"))?;
        if value > 15 {
            return Err(format!("concept axis {idx} value {value} exceeds 15"));
        }
        out[idx] = value as u8;
    }
    Ok(Concept6D::new(
        out[0], out[1], out[2], out[3], out[4], out[5],
    ))
}

fn concept_json(concept: Concept6D) -> Value {
    json!([
        concept.domain,
        concept.subdomain,
        concept.operation,
        concept.modality,
        concept.depth,
        concept.polarity
    ])
}

fn required_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string argument '{key}'"))
}

fn json_rpc_error(id: Value, code: i32, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message,
        },
        "id": id,
    })
}

pub fn pack_concept(concept: Concept6D) -> u32 {
    ((concept.domain as u32) << 20)
        | ((concept.subdomain as u32) << 16)
        | ((concept.operation as u32) << 12)
        | ((concept.modality as u32) << 8)
        | ((concept.depth as u32) << 4)
        | concept.polarity as u32
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn zymatica_wasm_alloc(len: usize) -> *mut u8 {
    let mut bytes = Vec::with_capacity(len);
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    ptr
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zymatica_wasm_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, 0, len);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zymatica_wasm_handle_json(ptr: *const u8, len: usize) -> *mut u8 {
    let input = unsafe { std::slice::from_raw_parts(ptr, len) };
    let response = match std::str::from_utf8(input) {
        Ok(text) => handle_edge_json(text),
        Err(err) => {
            json_rpc_error(Value::Null, -32700, format!("input is not UTF-8: {err}")).to_string()
        }
    };
    wasm_response_buffer(response.as_bytes())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zymatica_wasm_project_concept(ptr: *const u8, len: usize) -> u32 {
    let input = unsafe { std::slice::from_raw_parts(ptr, len) };
    let text = std::str::from_utf8(input).unwrap_or("");
    pack_concept(project_text_to_concept(text))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zymatica_wasm_rag_best(
    query_ptr: *const u8,
    query_len: usize,
    docs_ptr: *const u8,
    docs_len: usize,
) -> u32 {
    let query = unsafe { std::slice::from_raw_parts(query_ptr, query_len) };
    let docs = unsafe { std::slice::from_raw_parts(docs_ptr, docs_len) };
    let query = std::str::from_utf8(query).unwrap_or("");
    let docs = std::str::from_utf8(docs).unwrap_or("");
    let index = ConceptRagIndex::from_paragraphs(docs.lines());
    index
        .query(query, 1)
        .first()
        .map(|hit| hit.id as u32)
        .unwrap_or(u32::MAX)
}

#[cfg(target_arch = "wasm32")]
fn wasm_response_buffer(response: &[u8]) -> *mut u8 {
    let total_len = response.len() + 4;
    let ptr = zymatica_wasm_alloc(total_len);
    unsafe {
        let out = std::slice::from_raw_parts_mut(ptr, total_len);
        out[..4].copy_from_slice(&(response.len() as u32).to_le_bytes());
        out[4..].copy_from_slice(response);
    }
    ptr
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WasmGpuBridge {
    pub buffer_id: u64,
    pub element_count: usize,
    pub element_size_bytes: usize,
    pub pointer: usize,
}

impl WasmGpuBridge {
    pub fn new(buffer_id: u64, element_count: usize, element_size_bytes: usize) -> Self {
        let buffer = vec![0u8; element_count * element_size_bytes];
        let pointer = buffer.as_ptr() as usize;
        std::mem::forget(buffer);
        Self {
            buffer_id,
            element_count,
            element_size_bytes,
            pointer,
        }
    }

    pub fn total_bytes(&self) -> usize {
        self.element_count * self.element_size_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_json_runs_concept_rag_tool() {
        let req = json!({
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "concept_rag",
                "arguments": {
                    "query": "solar energy output",
                    "documents": ["solar panel efficiency", "wind turbine velocity", "hydroelectric flow"]
                }
            }
        })
        .to_string();
        let resp = handle_edge_json(&req);
        assert!(resp.contains("solar panel efficiency"));
    }

    #[test]
    fn edge_json_runs_set_s_tool() {
        let req = json!({
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "set_s_select",
                "arguments": {
                    "branches": [
                        {"branch_id": 1, "parent_branch_id": 0, "tokens": [10, 20], "confidence": 0.9},
                        {"branch_id": 2, "parent_branch_id": 0, "tokens": [30, 40], "confidence": 0.5}
                    ],
                    "target_tokens": [10, 20]
                }
            }
        })
        .to_string();
        let resp = handle_edge_json(&req);
        assert!(resp.contains("selected_tokens"));
    }

    #[test]
    fn test_wasm_gpu_bridge_zero_copy_alloc() {
        let bridge = WasmGpuBridge::new(101, 1024, 4);
        assert_eq!(bridge.total_bytes(), 4096);
        assert!(bridge.pointer > 0);
    }
}
