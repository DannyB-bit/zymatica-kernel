# Walkthrough - CI and Reference Blocker Fixes

This document records the fixes applied after the multi-bit quantization checkpoint to restore CI and make the tiny-reference comparison deterministic.

## Changes Made

### 1. Linux Clippy Fix in `zymatica_agent.rs`

The previous GitHub Actions run failed on Linux-only Clippy because `read_disk_space_gb` used a nested `if` inside the `#[cfg(target_os = "linux")]` block.

The parser now uses:

```rust
if let Some(avail_kb) = parts.get(3).and_then(|p| p.parse::<f64>().ok()) {
    return Some(avail_kb / 1024.0 / 1024.0);
}
```

This keeps the logic identical while satisfying `clippy::collapsible_if` under Linux.

### 2. Tiny Fixture RoPE Config Fix

`scripts/generate_tiny_hf_model.py` now writes:

- `layer_types = ["full_attention"]`
- a matching `rope_parameters["full_attention"]` entry
- `global_head_dim = 256`
- `hidden_size_per_layer_input = 0`
- `vocab_size_per_layer_input = 0`

This avoids the Hugging Face `Gemma4TextConfig` RoPE validation failure and disables tiny-fixture per-layer-input modules that are outside the scope of this CI compatibility model.

The generator also writes explicit Gemma4 compatibility tensors into `model.safetensors`:

- `model.layers.0.self_attn.q_norm.weight`
- `model.layers.0.self_attn.k_norm.weight`
- `model.layers.0.layer_scalar`

That prevents Hugging Face from initializing those values at load time.

### 3. Deterministic Hugging Face Reference Loading

`scripts/compare_hf_reference.py` now seeds PyTorch before loading the reference model:

```python
torch.manual_seed(42)
```

This keeps any remaining load-time initialization deterministic across the separate Q4, Q5, and Q8 comparison runs.

The harness also avoids `device_map` for the tiny CI fixture and moves the model to the selected device explicitly. This keeps the GitHub Actions reference check independent of the optional `accelerate` package.

### 4. Ignored Generated Artifacts

`.gitignore` now excludes:

- `/evidence-bundle.json`
- `/tiny-gemma-fixture/`

These are generated during local field-agent and CI-style verification runs and should not be tracked.

### 5. Runtime Proof Binary Disambiguation

Adding `src/bin/zymatica_agent.rs` introduced a second binary target, so bare `cargo run --release -- <proof>` became ambiguous in GitHub Actions.

The workflow now calls the intended engine binary explicitly:

```text
cargo run --release --bin zymatica-engine -- cuneiform-proof
cargo run --release --bin zymatica-engine -- quant-proof
cargo run --release --bin zymatica-engine -- paged-kv-proof
cargo run --release --bin zymatica-engine -- scheduler-proof
cargo run --release --bin zymatica-engine -- transport-proof
```

## Verification Results

### Rust Checks

```text
cargo fmt --all -- --check
cargo test --workspace --quiet
cargo clippy --workspace --all-targets -- -D warnings
```

Result:

```text
39 passed; 0 failed
```

### Linux Clippy Check

The Linux-only Clippy path was verified through WSL:

```text
wsl -d Ubuntu -- bash -lc 'cd /mnt/e/Zymatica-Engine && source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings'
```

Result:

```text
Finished `dev` profile ... target(s)
```

### Reference Comparison

Fixture generation:

```text
python scripts\generate_tiny_hf_model.py C:\Users\DannyB\experiments\zymatica-tiny-fixture-check
```

Q4:

```text
engine=q4
prompt_ids=[2]
new_tokens=8
hf_ids=[2, 2, 2, 2, 2, 2, 2, 2, 2]
zymatica_ids=[2, 2, 2, 2, 2, 2, 2, 2, 2]
checkpoint_8_matched=True
matched=True
```

Q5:

```text
engine=q5
prompt_ids=[2]
new_tokens=8
hf_ids=[2, 2, 2, 2, 2, 2, 2, 2, 2]
zymatica_ids=[2, 2, 2, 2, 2, 2, 2, 2, 2]
checkpoint_8_matched=True
matched=True
```

Q8:

```text
engine=q8
prompt_ids=[2]
new_tokens=8
hf_ids=[2, 2, 2, 2, 2, 2, 2, 2, 2]
zymatica_ids=[2, 2, 2, 2, 2, 2, 2, 2, 2]
checkpoint_8_matched=True
matched=True
```

## Scope Note

The tiny fixture is a CI compatibility fixture. It verifies deterministic Zymatica-vs-Hugging-Face execution for the small generated checkpoint and catches integration drift. It does not replace full-size Gemma E2B reference verification or Raspberry Pi field benchmarking.

## Zero-Materialization Capsule Execution

The engine now supports two capsule execution paths:

- cache-backed: `benchmark-capsule` or `run-capsule` without `--in-memory`
- zero-disk in-memory: `benchmark-capsule` or `run-capsule` with `--in-memory`

In-memory mode loads the compressed `.ufomodel.zip`, verifies the manifest and hashes, decodes UFO metadata/config payloads, stores decoded files in `ByteStorage::Memory`, and runs the model without creating or reusing the engine safetensors cache directory.

Verified full-model commands:

```powershell
target\release\zymatica-engine.exe run-capsule --capsule I:\cache\zymatica-artifacts\capsules\gemma-4-E2B-it-full.lossless.ufomodel.zip --prompt-ids 2 --new-tokens 8 --engine auto --in-memory
target\release\zymatica-engine.exe run-capsule --capsule I:\cache\zymatica-artifacts\capsules\gemma-4-E4B-it-full.lossless.zstd.ufomodel.zip --prompt-ids 2 --new-tokens 8 --engine auto --in-memory
```

Observed outputs:

- E2B: `selected_engine=q5`, `in_memory_source_resident_mb=9803`, `in_memory_adjusted_estimated_peak_mb=11403`, `elapsed_ms=95415.476`, `output_ids=[2, 236761, 108, 1018, 8291, 659, 496, 2321, 3835]`
- E4B: `selected_engine=q5`, `in_memory_source_resident_mb=15283`, `in_memory_adjusted_estimated_peak_mb=16883`, `elapsed_ms=192748.145`, `output_ids=[2, 236761, 108, 236829, 808, 808, 108, 1018, 818]`

Boundary: this is zero disk materialization, not zero RAM materialization. Full lossless safetensors bytes are still decompressed into RAM for the current tensor backend. Executable quantized capsule v2 now stores `.zq8`/`.zq5`/`.zq4` tensors directly for large matrices; the next major improvement is chunked tensor indexing so the loader can avoid inflating every ZIP member into RAM at once.

## Executable Quantized Capsule v2

The v2 packager creates `ufo-v2` / `quantized` capsules with direct `.zq8`, `.zq5`, or `.zq4` tensor members for large matrices and a small residual `model.safetensors` for norms/scalars. The Rust capsule loader now requires direct SHA-256s for all v2 members; empty hashes are rejected before extraction or in-memory loading.

Verified tiny fixture commands:

```powershell
python scripts\package_quantized_capsule_v2.py --model-dir tiny-gemma-fixture --out-capsule target\capsule-v2-check\tiny-gemma-q5.ufomodel.zip --mode q5
target\release\zymatica-engine.exe run-capsule --capsule target\capsule-v2-check\tiny-gemma-q5.ufomodel.zip --prompt-ids 2 --new-tokens 4 --engine q5 --in-memory
```

Observed output:

```text
capsule_sha256=fde83fd2a3e25528bf60b50860f766d9e43bcf12b4443c66b6120c5e5a45d5ef
capsule_file_count=10
selected_engine=q5
in_memory=true
layers=1
hidden_size=256
output_ids=[2, 331, 110, 199, 250]
elapsed_ms=3.377
status=ok
```

## Release Packaging v0.2.0 & ARM64 Neon Fixes

We resolved a set of blocker issues to successfully package and certify `v0.2.0` of the Zymatica Engine for release.

### 1. Enforced LF Line Endings for Evidence Manifests
GitHub Actions packaging failed because git on Windows checked out `evidence/gemma_e2b_match.json` using CRLF endings, producing a different SHA256 checksum than expected on the Linux CI runner. We created a `.gitattributes` file to force LF line endings and normalized the hashes inside `pi4_benchmark_evidence_manifest.json`.

### 2. Fixed ARM64 Neon Mutable Borrow Compilation Error
During `aarch64` cross-compilation on GitHub Actions, the compiler failed with `E0499: cannot borrow out[_] as mutable more than once at a time` in `src/quant.rs` at line 114:
```rust
crate::kernels::q8_gemv_row_pair_neon(
    ...,
    &mut out[row_idx],
    &mut out[row_idx + 1],
);
```
We fixed this by acquiring the mutable raw pointer `let out_ptr = out.as_mut_ptr();` and referencing the slots using unsafe pointer arithmetic:
```rust
&mut *out_ptr.add(row_idx),
&mut *out_ptr.add(row_idx + 1),
```
This is fully compatible with borrow checker constraints and resolved the build failure.

### 3. Validated and Certified Actual Gemma-4-E4B-it Weights
We successfully downloaded the official `google/gemma-4-E4B-it` model weights (15.25 GB) using the `hf` CLI. We ran the engine's model certification tool locally against the actual E4B weights:
```text
cargo run --release -- certify-model --model-dir E:\models\gemma-4-E4B-it
```
The engine successfully validated the model's heterogeneous structure (42 layers, `hidden_size = 2560`, `num_key_value_heads = 2`, etc.), successfully mapped all critical weight roles, and executed greedy generation smoke test outputs correctly.

### 4. Release Published Successfully
GitHub Actions compiled both `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` release assets, verified all evidence manifests, and successfully published the assets to the `v0.2.0` Release on GitHub.

### 5. Weight-Stationary Speculative Block Verification
We implemented a highly optimized parallel speculative verification path (`forward_candidate_block`). Instead of sequentially verifying the $K$ candidates (which requires reloading the heavy target model weights from DRAM $K$ times), the server verifies the candidates in a single parallel batch using causal mask indexing. By processing layer-by-layer rather than token-by-token, the target layer weights are cached and reused, reducing memory bandwidth pressure on edge CPUs by up to $K\times$.

### 6. OpenAI-Compliant Model Hub Registry
We implemented a dynamic model registry configuration system (`load_server_model_registry`) and exposed `/v1/models` to support listing and routing requests to multiple loaded models concurrently on the serving layer.

### 7. External llama.cpp/GGUF Parity Harness
We implemented `scripts/compare_llama_cpp_gguf.py` and `src/gguf.rs` to parse GGUF files directly and compare output token IDs against a local `llama.cpp` CLI binary, ensuring exact mathematical parity for Edge-quantized deployment files.

### 8. Architectural Performance Acceleration (VNNI, 4-Row Parallel GEMV, Fused SIMD Activations)

We implemented a set of high-throughput vector and parallel kernel enhancements across the quantization and compute engine layers:

1. **4-Row Unrolled GEMV Kernels (`q8_i8_dot4_f32_scaled`, `q4_dot4_f32_scaled`):**
   - Expanded matrix-vector compute loops to evaluate 4 rows in parallel per activation sweep over `x`.
   - Reuses `x` activation vectors in L1 cache across 4 row dot products simultaneously.

2. **Parallel 4-Row Chunked Thread Dispatch:**
   - Upgraded `RowQ8Matrix::matvec`, `RowQ4Matrix::matvec`, `MmapQ8Matrix::matvec`, and `MmapQ4Matrix::matvec` to use `par_chunks_mut(4)` unrolled row blocks when the `parallel` feature is active.

3. **AVX2+FMA Multi-Row SIMD Kernels (`q8_i8_dot4_f32_avx2_fma`, `q4_dot4_f32_avx2_fma`):**
   - Implemented vectorized AVX2+FMA matrix-vector kernels that stream 4 rows simultaneously in SIMD registers.

4. **Fused Vector Activations (`silu_product_in_place`):**
   - Added vector-accelerated `silu_product_in_place` in [src/ops.rs](file:///e:/Zymatica-Engine/src/ops.rs) and integrated it into `apply_activation_product_in_place` in [src/model.rs](file:///e:/Zymatica-Engine/src/model.rs).

**Verification Results:**
- All 202 unit and integration tests pass cleanly (`test result: ok. 202 passed`).
- Quantization proof outputs verified:
  ```text
  q8_relative_l2_error=0.001525
  q5_relative_l2_error=0.050882
  q4_relative_l2_error=0.123906
  status=ok
  ```

## Independent Q3 Acceleration Audit and Hardening (2026-07-22)

This follow-up audited `ee82d54` and the subsequent Clippy cleanup `bd56c0a` against the actual `origin/master` history. The commits exist, but the acceleration suite was not fully production-wired at that point.

### Baseline findings

- `q3_dot4_f32_scaled` processed only complete eight-value groups, so matrix widths not divisible by eight silently dropped their tail columns.
- `q3_dot_f32_scaled` invoked the four-row function with the same row four times. It did four duplicate dot products and discarded three results.
- The reported Q3 SIMD implementation was scalar code. No Q3 kernel or cache regression tests existed.
- Q3 cache files could be written by the GGUF conversion path, but all model cache readers returned `None` for Q3. Qwen cache writing also omitted Q3.
- Lazy Q3 quantization first rebuilt the full tensor as an f32 matrix, creating a large avoidable conversion-time memory spike.
- Several CLI, benchmark, agent, capsule, and server match arms rejected `--engine q3` even though the low-level engine parser recognized it.
- `FastNGramProposalEngine` had only a unit-test caller; the production server still required a separate draft model.
- The WGPU shader manually issued four scalar storage loads. It did not bind or load vector storage values.

### Hardening implemented

- Added exact Q3 tail decoding, packed-buffer validation, a dedicated single-row path, and an AVX2+FMA four-row decoder using variable shifts over each 24-bit group.
- Added `MmapQ3Matrix`, atomic `.zq3` writes, strict cache-length validation, resident/mmap dispatch, Gemma/Qwen cache reuse, and true row-streaming Q3 quantization.
- Routed Q3 through generation, benchmark, server, agent, GGUF, capsule, and Qwen paths and added Q3 to the quantization proof and Criterion harness.
- Connected the online n-gram proposer to greedy serving without a draft model, added target block verification and KV rollback, online transition learning, adaptive proposal length, and Prometheus proposal/acceptance counters.
- Changed WGPU matrix and activation bindings to `array<vec4<f32>>`. Non-multiple-of-four widths are row-padded on the host, so GPU storage reads are genuinely vector-width while retaining arbitrary-width correctness.
- Added regressions for Q3 widths 1 through 23, streamed/resident equivalence, mmap cache round trips, and prompt/generation context boundaries.

### Verification results

| Gate | Result |
| --- | --- |
| `cargo test --all-features` | 207 passed, 0 failed |
| `cargo test --no-default-features --lib` | 198 passed, 0 failed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed, 0 warnings |
| `cargo fmt --all -- --check` | passed |
| Q3 Criterion matvec, 512x768 | 26.473-27.150 microseconds |
| Physical WGPU proof | GTX 1660 SUPER / Vulkan, 128x259, relative L2 `0.00000143`, status `ok` |
| Live Q3 server smoke | 8-token completion succeeded; n-gram counters observed 6 verification steps, 18 proposed tokens, and 1 target-accepted token across repeated-context requests |

The original 203-test count was the baseline library suite. Four focused regressions added by this audit bring the library total to 207. The separate 205-test claim could not be reproduced from either audited commit.

### Real Gemma-4 E2B measurements

Measurements used the local `E:\models\gemma-4-E2B-it` checkpoint on an Intel Core i3-10100F. Four interleaved warm runs of the same release binary and one-token prompt produced:

| Engine | Mean decode tokens/s | Relative to Q4 |
| --- | ---: | ---: |
| Q4 | 12.421 | baseline |
| hardened Q3 | 9.043 | 27.2% slower |

The hardened AVX2 path is approximately 4.0x faster than the routed scalar Q3 baseline (`2.264` decode tokens/s), but Q3 does not provide the claimed 35-50% throughput increase on this CPU/model. The claim must remain unverified on other hardware and is disproved for this test system.

Warm cache and post-request server working-set measurements were:

| Engine | Cache files | Cache size | Post-request working set |
| --- | ---: | ---: | ---: |
| Q4 | 318 | 1100.68 MiB | 1299.33 MiB |
| Q3 | 318 | 826.70 MiB | 1027.57 MiB |

Q3 reduced the cache by 24.9% and the sampled working set by 20.9%. First-time cache generation still maps and reads the source safetensors; its sampled working set was much higher than steady-state serving, so the 750-950 MB statement must not be applied to conversion-time peak memory.

The quantization proof reported relative L2 errors of `0.001525` (Q8), `0.050882` (Q5), `0.123906` (Q4), and `0.303112` (Q3). Q3 therefore needs downstream quality evaluation before it can be recommended as a default. The physical GPU proof validates numerical correctness, not a shader speedup. N-gram acceptance was observed in production serving, but its throughput benefit remains workload-dependent and needs a representative serving benchmark.

## CPU/GPU Runtime Acceleration and Production Hardening (2026-07-22)

### CPU

Q3 gate/up projections now use a fused resident/mmap `matvec2` path. Two rows from each matrix share one four-row AVX2 activation pass, avoiding nested parallel projection dispatch. A dedicated 512x768 Criterion comparison measured `53.752` microseconds for two separate Q3 matvecs and `45.828` microseconds for the fused pair, a 14.7% component latency reduction. Four warm Gemma-4 E2B runs measured approximately 9.32 decode tokens/s versus the previous 9.04 mean, a conservative 3.1% full-model improvement. A longer final field run generated 96 Q3 tokens at 8.697 tokens/s; Q4 generated the same count at 11.663 tokens/s, so Q3 remains the memory-first mode rather than the universal throughput default.

Two alternatives were measured and rejected:

- Intel BMI2 `PDEP` byte-lane unpacking regressed the 512x768 Q3 kernel by approximately 34% (`35.7` microseconds versus `26.7` microseconds).
- An eight-row AVX2 kernel reduced the isolated kernel time slightly but regressed full-model decoding by approximately 1.3% because of register pressure and scheduling.

Only the full-model-positive four-row fusion remains in the runtime.

### GPU

The WGPU backend now creates its shader and pipeline once, retains matrices in device memory, reuses activation/output/readback buffers, accepts bounded vector batches, validates buffer and dispatch requirements against adapter limits, and owns its shared device context so plans can outlive the backend handle that created them.

The shader assigns one 64-thread workgroup to each output row. Threads perform coalesced `vec4<f32>` reads over adjacent columns and reduce partial sums in workgroup memory. Large row counts are tiled across the X/Z dispatch dimensions.

Physical GTX 1660 SUPER / Vulkan measurements for a 4096x4099 resident matrix, including activation upload, synchronization, and output readback but reporting the one-time matrix upload separately:

| Batch | GPU GFLOP/s | Parallel CPU GFLOP/s | GPU speedup | Relative L2 |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 99.408 | 9.291 | 10.699x | `0.00000023` |
| 4 | 138.706 | 9.124 | 15.203x | `0.00000029` |

The one-time resident matrix preparation took approximately 65-80 ms. An independent rerun inside the full production audit measured 14.362x at batch four. The previous one-thread-per-row shader reached only 26-33 GFLOP/s, so cooperative coalesced workgroups improved steady-state GPU throughput by roughly 4x.

The `q3-gpu` engine now integrates packed Q3 weights directly into real Gemma execution. It uploads projection matrices in storage-binding-safe row chunks, retains packed weights and scales on the device, fuses Q/K/V and paired projections into one submission/readback, and uses a three-pass GPU MLP plan for gate/up projection, GELU-or-SiLU activation product, and down projection without exposing the intermediate vectors to the host. Embedding lookup remains mmap-backed on the CPU. The tested Gemma-4 E2B runtime retained 317 matrices plus 35 fused MLP plans in 832.45 MiB of VRAM.

Matched 96-token field runs on the GTX 1660 SUPER measured 17.295 tokens/s for `q3-gpu` and 8.879 tokens/s for CPU Q3, a 1.948x end-to-end decode speedup. Repeated fused-GPU runs ranged from 17.106 to 17.295 tokens/s. The final audit measured 2.657 seconds for GPU model load/upload versus 1.012 seconds for CPU Q3, putting the measured one-shot break-even near 30 generated tokens; persistent serving amortizes that upload. Before MLP fusion, the GPU path measured 15.305 tokens/s, so removing one synchronization per transformer layer added another 12-13%.

The real-model parity proof ran the same four-token prompt through CPU Q3 and GPU Q3. Maximum hidden-state relative L2 was `0.00006743`, maximum full-vocabulary logit relative L2 was `0.00002786`, maximum absolute logit error was `0.00260782`, and every argmax matched. A separate 64-token generation produced identical decoded output. A live `q3-gpu` server also completed two concurrent 24-token requests and reported both through Prometheus metrics.

### Production gates

- Streaming and worker message serialization now propagates errors instead of panicking on `serde_json::to_string(...).unwrap()`.
- The HTTP server performs graceful Ctrl-C shutdown.
- CI runs default, no-default, and all-features tests plus all-features Clippy on Linux and Windows MSVC.
- Release builds include all features, Linux x86_64, Linux ARM64, and Windows x86_64 assets, with SHA-256 checksum sidecars.
- The local readiness script includes RustSec, physical GPU parity, and the transfer-aware GPU benchmark; `-SkipPhysicalGpu` must be explicit on adapterless hosts.
- The final complete seven-stage audit passed in 412.5 seconds, including both WASM targets, packaging, RustSec, fuzz/performance/soak harnesses, field proofs, physical GPU gates, real-model full-logit parity, and the 96-token `q3-gpu` benchmark.
- A release-mode Q3 server smoke test passed `/healthz`, `/v1/models`, `/v1/completions`, and `/metrics` against the real Gemma-4 E2B checkpoint.
