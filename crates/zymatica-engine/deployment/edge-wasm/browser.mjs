export async function createZymaticaEngine(wasmUrl) {
  const bytes = await fetch(wasmUrl).then((response) => {
    if (!response.ok) {
      throw new Error(`failed to fetch ${wasmUrl}: ${response.status}`);
    }
    return response.arrayBuffer();
  });
  const instance = await instantiateWasm(bytes);
  return createClient(instance.exports);
}

export async function instantiateWasm(moduleOrBytes) {
  const result = await WebAssembly.instantiate(moduleOrBytes, {});
  return result.instance ?? result;
}

export function createClient(exports) {
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  function writeString(text) {
    const bytes = encoder.encode(text);
    const ptr = exports.zymatica_wasm_alloc(bytes.length);
    new Uint8Array(exports.memory.buffer, ptr, bytes.length).set(bytes);
    return { ptr, len: bytes.length };
  }

  function readResponse(ptr) {
    const header = new DataView(exports.memory.buffer, ptr, 4);
    const len = header.getUint32(0, true);
    const bytes = new Uint8Array(exports.memory.buffer, ptr + 4, len);
    const text = decoder.decode(bytes);
    exports.zymatica_wasm_dealloc(ptr, len + 4);
    return JSON.parse(text);
  }

  function call(payload) {
    const input = writeString(JSON.stringify(payload));
    const outPtr = exports.zymatica_wasm_handle_json(input.ptr, input.len);
    exports.zymatica_wasm_dealloc(input.ptr, input.len);
    return readResponse(outPtr);
  }

  return {
    call,
    tools() {
      return call({ jsonrpc: "2.0", method: "tools/list", id: 1 });
    },
    conceptRag(query, documents, limit = 1) {
      return call({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
          name: "concept_rag",
          arguments: { query, documents, limit },
        },
        id: 2,
      });
    },
  };
}
