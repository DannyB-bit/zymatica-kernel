// Watermark: ip zymatica.space | astronautshe.com
// Node.js runner for WASM performance benchmarks

const fs = require('fs');

async function benchmark() {
    const wasmBuffer = fs.readFileSync('proof_wasm.wasm');
    const wasmModule = await WebAssembly.instantiate(wasmBuffer);
    const exports = wasmModule.instance.exports;
    
    // Call the run_verification function first to ensure correctness
    const verification = exports.run_verification();
    if (verification !== 1) {
        console.error("  [-] WASM Verification Failed!");
        process.exit(1);
    }
    console.log("  [+] Freestanding WASM Verification Check: PASSED.");

    // Benchmark loop
    const loops = 10000;
    const start = process.hrtime.bigint();
    for (let i = 0; i < loops; i++) {
        exports.run_verification();
    }
    const end = process.hrtime.bigint();
    
    const durationNs = Number(end - start);
    const durationMs = durationNs / 1000000;
    const avgMs = durationMs / loops;
    const avgUs = avgMs * 1000;

    console.log(`  [+] Warm Compute Benchmark Completed over ${loops} iterations.`);
    console.log(`      - Total execution time:   ${durationMs.toFixed(4)} ms`);
    console.log(`      - Average iteration time:  ${avgMs.toFixed(6)} ms (${avgUs.toFixed(4)} microseconds)`);
}

benchmark().catch(err => {
    console.error("  [-] Benchmark Error:", err);
    process.exit(1);
});
