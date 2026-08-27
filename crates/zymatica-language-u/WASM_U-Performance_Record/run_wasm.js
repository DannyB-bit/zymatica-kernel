// Watermark: ip zymatica.space | astronautshe.com
// Node.js WebAssembly FFI Integration Script

const fs = require('fs');

async function run() {
    // 1. Read input coordinates
    const testInput = JSON.parse(fs.readFileSync('test_input.json', 'utf8'));
    const count = testInput.length;

    // 2. Load and compile WASM
    const wasmBuffer = fs.readFileSync('proof_wasm.wasm');
    const wasmModule = await WebAssembly.instantiate(wasmBuffer);
    const exports = wasmModule.instance.exports;
    const memory = exports.memory;

    // 3. Coordinate Structure sizes:
    // Concept6D has 6 fields of u8 (size = 6 bytes)
    const structSize = 6;
    const inputOffset = 102400;
    const memView = new Uint8Array(memory.buffer);

    for (let i = 0; i < count; i++) {
        const idx = inputOffset + i * structSize;
        const c = testInput[i];
        memView[idx] = c.domain;
        memView[idx + 1] = c.subdomain;
        memView[idx + 2] = c.operation;
        memView[idx + 3] = c.modality;
        memView[idx + 4] = c.depth;
        memView[idx + 5] = c.polarity;
    }

    // 4. Invoke wasm_encode
    const encodedBufferPtr = exports.wasm_encode(inputOffset, count);
    const encodedBitsCount = exports.wasm_get_encoded_bits();
    const encodedBytesCount = Math.floor((encodedBitsCount + 7) / 8);

    // Read back the compressed payload bytes
    const encodedBytes = new Uint8Array(memory.buffer, encodedBufferPtr, encodedBytesCount);
    fs.writeFileSync('payload_wasm.bin', Buffer.from(encodedBytes));

    // 5. Invoke wasm_decode
    const decodedBufferPtr = exports.wasm_decode(encodedBufferPtr, encodedBytesCount, count);

    // Read back the decoded Concept6D structs from WASM memory
    const decodedView = new Uint8Array(memory.buffer, decodedBufferPtr, count * structSize);
    const decodedConcepts = [];

    for (let i = 0; i < count; i++) {
        const idx = i * structSize;
        decodedConcepts.push({
            domain: decodedView[idx],
            subdomain: decodedView[idx + 1],
            operation: decodedView[idx + 2],
            modality: decodedView[idx + 3],
            depth: decodedView[idx + 4],
            polarity: decodedView[idx + 5]
        });
    }

    fs.writeFileSync('test_output_wasm.json', JSON.stringify(decodedConcepts, null, 2));
    console.log(`  [+] Node.js completed WASM execution. Encoded ${count} concepts into ${encodedBitsCount} bits.`);
}

run().catch(err => {
    console.error("  [-] Node.js Error:", err);
    process.exit(1);
});
