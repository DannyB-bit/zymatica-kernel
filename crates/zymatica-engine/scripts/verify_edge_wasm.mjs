import { readFile } from "node:fs/promises";
import { createClient } from "../deployment/edge-wasm/browser.mjs";

const wasmPath =
  process.argv[2] ??
  "target/wasm32-unknown-unknown/release/zymatica_core.wasm";

const bytes = await readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes, {});
const engine = createClient(instance.exports);

const tools = engine.tools();
if (tools.result.tools.length !== 4) {
  throw new Error(`expected 4 edge tools, got ${tools.result.tools.length}`);
}

const rag = engine.conceptRag("solar panel status", [
  "Solar array power output is normal.",
  "Reservoir water flow is nominal.",
]);
if (rag.result.hits[0].id !== 0) {
  throw new Error(`concept_rag selected wrong hit: ${JSON.stringify(rag)}`);
}

const setS = engine.call({
  jsonrpc: "2.0",
  method: "tools/call",
  params: {
    name: "set_s_select",
    arguments: {
      target_tokens: [20, 22],
      branches: [
        { tokens: [10, 11], logprob: -0.01 },
        { tokens: [20, 22], logprob: -0.2 },
      ],
    },
  },
  id: 3,
});
if (setS.result.accepted_prefix_len !== 2) {
  throw new Error(`set_s_select failed: ${JSON.stringify(setS)}`);
}

const mask = engine.call({
  jsonrpc: "2.0",
  method: "tools/call",
  params: {
    name: "concept_mask_count",
    arguments: {
      vocab_size: 256,
      min: [0, 0, 0, 0, 0, 0],
      max: [15, 15, 15, 15, 15, 15],
    },
  },
  id: 4,
});
if (mask.result.allowed !== 256) {
  throw new Error(`concept_mask_count failed: ${JSON.stringify(mask)}`);
}

console.log("runtime=zymatica-engine");
console.log("mode=edge-wasm-node-instantiation-proof");
console.log(`wasm_path=${wasmPath}`);
console.log(`tools=${tools.result.tools.length}`);
console.log(`rag_hit_id=${rag.result.hits[0].id}`);
console.log(`set_s_accepted_prefix_len=${setS.result.accepted_prefix_len}`);
console.log(`semantic_mask_allowed=${mask.result.allowed}`);
console.log("status=ok");
