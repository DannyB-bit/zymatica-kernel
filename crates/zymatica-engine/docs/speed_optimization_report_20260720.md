# Zymatica Inference Speed Optimization Report - 2026-07-20

This report records real local inference benchmarks from the Gemma and Qwen quantized runtime speed passes.

## Host

- CPU: Intel Core i3-10100F, 4 cores / 8 logical processors
- OS: Windows
- Rust: 1.90.0, MSVC target
- Models: local Gemma E2B/E4B HF checkpoints and local Qwen3.5 0.8B checkpoint
- Fastest verified engine path: Q4 cached mmap

## Command

```powershell
target\release\zymatica-engine.exe benchmark-inference `
  --model-dir E:\models\gemma-4-E2B-it `
  --prompt-ids 2 `
  --new-tokens 16 `
  --engine q4 `
  --q8-cache-dir E:\models\gemma-4-E2B-it\.zymatica-cache-q4
```

## Verified Results

The output token IDs remained stable across the optimization pass:

```text
[2, 236771, 236795, 3677, 236761, 6639, 236769, 3677, 236761, 6639, 236769, 3677, 236761, 6639, 236769, 3677, 236761]
```

Measured Q4 decode throughput improved from the earlier baseline of about `10.52 tok/s` to a verified current band of about `13.02-13.15 tok/s` on this host. Cold total time for the 16-token benchmark is now about `2.41-2.45s` after the model cache is present.

Representative current runs:

| Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| Q4 mmap | 16 | 1118.220 | 1145.558 | 13.094060 | ok |
| Q4 mmap | 16 | 1101.952 | 1151.963 | 13.021250 | ok |
| Q4 mmap | 16 | 1086.784 | 1140.420 | 13.153050 | ok |
| Q5 mmap | 8 | 1871.131 | 970.528 | 7.212572 | ok |

Additional real-model compressed runs after Qwen cache support landed:

| Model | Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Status |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Gemma E2B | Q4 mmap | 16 | 1553.637 | 1178.111 | 12.732247 | ok |
| Gemma E4B | Q4 mmap | 4 | 1408.566 | 446.086 | 6.725155 | ok |
| Qwen3.5 0.8B | f32 lazy | 4 | 220.725 | 582.392 | 5.151171 | ok |
| Qwen3.5 0.8B | Q4 mmap | 4 | 239.179 | 107.970 | 27.785393 | ok |
| Qwen3.5 0.8B | Q5 mmap | 4 | 578.511 | 171.886 | 17.453479 | ok |

Fused pair-projection pass, measured on 2026-07-20:

| Model | Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Delta vs Prior Local Baseline | Status |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Gemma E2B | Q4 mmap | 16 | 1084.066 | 1144.653 | 13.104405 | +2.9% vs 12.732247 | ok |
| Gemma E4B | Q4 mmap | 4 | 1399.392 | 431.888 | 6.946245 | +3.3% vs 6.725155 | ok |
| Qwen3.5 0.8B | Q4 mmap | 4 | 247.958 | 107.255 | 27.970750 | +0.7% vs 27.785393 | ok |
| Gemma E2B | Q5 mmap | 8 | 1842.276 | 961.289 | 7.281891 | +3.6% vs 7.029919 | ok |
| Gemma E4B | Q5 mmap | 8 | 3041.629 | 1883.918 | 3.715661 | warm-cache baseline recorded | ok |
| Qwen3.5 0.8B | Q5 mmap | 4 | 1794.896 | 169.278 | 17.722337 | +1.5% vs 17.453479 | ok |

Implementation note: fused Q5 pair projection remains thresholded at `1536` columns. Q4 pair projection now fuses at `1024` columns after hoisting Q4 dot2 CPU/thermal dispatch out of the per-row path.

Gemma RMSNorm / in-place MLP pass, measured on 2026-07-20:

| Model | Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Delta vs Prior Local Baseline | Status |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Gemma E2B | Q4 mmap | 16 | 1105.112 | 1126.209 | 13.319020 | +1.6% vs 13.104405 | ok |
| Gemma E4B | Q4 mmap | 4 | 1401.232 | 429.371 | 6.986957 | +0.6% vs 6.946245 | ok |

Q4 planned-kernel dispatcher pass, measured on 2026-07-20:

| Model | Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Delta vs Prior Local Baseline | Status |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Gemma E2B | Q4 mmap | 16 | 1099.644 | 1116.678 | 13.432706 | +0.9% vs 13.319020 | ok |
| Gemma E4B | Q4 mmap | 4 | 1407.397 | 429.221 | 6.989398 | neutral vs 6.986957 | ok |
| Qwen3.5 0.8B | Q4 mmap | 16 | 240.267 | 540.036 | 27.775916 | +0.3% vs 27.680680 same-length baseline | ok |

Gemma hot-path env/early-exit cleanup pass, measured on 2026-07-20:

| Model | Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Delta vs Prior Local Baseline | Status |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Gemma E2B | Q4 mmap | 16 | 1087.718 | 1108.345 | 13.533692 | +0.8% vs 13.432706 | ok |
| Gemma E4B | Q4 mmap | 4 | 1390.982 | 430.374 | 6.970675 | neutral/noisy vs 6.989398; same-pass high 7.037742 | ok |
| Qwen3.5 0.8B | Q4 mmap | 16 | 244.984 | 546.049 | 27.470072 | unchanged Gemma-only path | ok |

Gemma per-layer input cache pass, measured on 2026-07-20:

| Model | Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Delta vs Prior Local Baseline | Status |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Gemma E2B | Q4 mmap | 16 | 1102.256 | 1101.158 | 13.622025 | +1.4% vs 13.432706 | ok |
| Gemma E4B | Q4 mmap | 4 | 1397.041 | 424.770 | 7.062646 | +1.0% vs 6.989398 | ok |

Q4 dot2 dispatch-hoist / 1024-wide fusion pass, measured on 2026-07-20:

| Model | Engine | Completion Tokens | Load ms | Decode ms | Decode tok/s | Delta vs Prior Local Baseline | Status |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Gemma E2B | Q4 mmap | 16 | 1095.147 | 1106.819 | 13.552354 | +0.9% vs 13.432706 planned-kernel baseline; within cache-pass band | ok |
| Gemma E4B | Q4 mmap | 4 | 1408.893 | 421.645 | 7.114985 | +1.8% vs 6.989398 planned-kernel baseline | ok |
| Qwen3.5 0.8B | Q4 mmap | 8 | 240.383 | 252.308 | 27.743825 | +0.8% vs 27.529845 same-length pre-pass | ok |

Real tokenizer-backed text executions also passed:

```powershell
target\release\zymatica-engine.exe generate --model-dir C:\Users\DannyB\qwen-3.5-0.8b-HEALED --tokenizer C:\Users\DannyB\qwen-3.5-0.8b-HEALED\tokenizer.json --prompt "Zymatica compressed inference speed test" --new-tokens 8 --engine q4 --q8-cache-dir C:\Users\DannyB\qwen-3.5-0.8b-HEALED\.zymatica-cache-q4 --temperature 0 --top-k 1
target\release\zymatica-engine.exe generate --model-dir E:\models\gemma-4-E2B-it --tokenizer E:\models\gemma-4-E2B-it\tokenizer.json --prompt "Zymatica compressed inference speed test" --new-tokens 8 --engine q4 --q8-cache-dir E:\models\gemma-4-E2B-it\.zymatica-cache-q4 --temperature 0 --top-k 1
target\release\zymatica-engine.exe generate --model-dir E:\models\gemma-4-E4B-it --tokenizer E:\models\gemma-4-E4B-it\tokenizer.json --prompt "Zymatica compressed inference speed test" --new-tokens 4 --engine q4 --q8-cache-dir E:\models\gemma-4-E4B-it\.zymatica-cache-q4 --temperature 0 --top-k 1
```

## Changes That Carried

- Cached RoPE application in native and quantized forward paths.
- Static graph instruction vectors cached on model construction.
- Mmap quant scale tables decoded once at load.
- Paired MLP gate/up projection scheduling for f32 and quantized Q4/Q5 paths.
- AVX2/FMA dot and softcap kernels.
- Lazy/capped RoPE trig-table construction for faster cold load.
- Expanded Q5 row layout to avoid per-token 5-bit unpacking.
- In-place RMS normalization for post-projection vectors.
- Vector-level activation dispatch for gated MLPs.
- Greedy benchmark argmax/NLL scan consolidation.
- Release `thin` LTO and single codegen unit.
- Qwen3.5 Q8/Q5/Q4 cached loader using the same `.zq*` matrix cache format as Gemma.
- Qwen3.5 paired/triple projection scheduling and cached partial RoPE.
- Resilient single-shard safetensors index fallback for local checkpoints whose index references a renamed split shard.
- Width-gated fused Q4 pair projection for MLP gate/up, reusing the activation vector across both projection rows on Gemma and Qwen 1024+ widths.
- Width-gated fused Q5 pair projection for expanded `i8` rows, reusing the activation vector across both projection rows on larger Gemma widths.
- AVX2 signed `i8 x i8` dot primitive for the existing Q8 dynamic-activation lane; this is tested but Q8i remains slower than Q8 on this host.
- AVX2/FMA weighted RMSNorm for x86 Gemma paths, with scalar and NEON fallbacks preserved.
- Gemma MLP activation product now reuses the gate buffer in place for `activation(gate) * up`, avoiding one intermediate allocation per MLP block.
- Q4 matvec selects the active thermal/CPU dot kernel once per matvec and reuses that plan across resident and mmap row loops, avoiding repeated per-row dispatch checks.
- Q4 matvec2 selects the active thermal/CPU dot kernel once per paired projection and reuses that plan across resident and mmap row loops, enabling the 1024-wide Qwen fused gate/up path without per-row feature checks.
- Gemma forward now caches `ZYMATICA_EARLY_EXIT_THRESHOLD` and `ZYMATICA_ATTENTION_SPARSITY` instead of hitting process environment lookup inside token/layer hot paths.
- Gemma early-exit checks now skip the variance helper entirely unless a concrete early-exit threshold is configured.
- Gemma forward now keeps a bounded 64-token per-model cache for token-derived per-layer inputs and stores cache entries behind `Arc`, so repeated token IDs reuse the expensive per-layer input projection without cloning the full vector set.
- Qwen3.5 MLP activation product now reuses the gate projection buffer in place for `silu(gate) * up`, avoiding one intermediate allocation per MLP block.

## Rejected Experiments

- Expanded Q4 hot layout (`ZYMATICA_Q4_HOT_UNPACK=1`) was slower on this host: `7.19 tok/s` decode because extra memory traffic outweighed decode savings.
- Fixed Rayon thread counts below default were slower: 1 thread `3.10 tok/s`, 2 threads `5.91 tok/s`, 4 threads `10.04 tok/s`, 6 threads `12.21 tok/s`, default about `12.86+ tok/s`.
- Public `q5i` engine exposure was rejected. It passed unit tests, but real E2B inference was slower than Q5-f32 activation: `6.212794 tok/s` vs `7.029919 tok/s` on the same 8-token benchmark.
- Unplanned per-row-dispatch Q4 pair fusion was rejected for Qwen3.5 0.8B because the 1024-wide path regressed from the prior `27.785393 tok/s` short baseline to `26.833199 tok/s`. The final implementation only enables Qwen-width Q4 fusion after selecting the dot2 kernel once per matvec2.
- Q/K/V K/V-pair scheduling fusion was rejected because it reduced parallelism on the 4-core host: E2B Q4 dropped to `12.963194 tok/s` and E4B Q4 dropped to `6.905635 tok/s`.
- Qwen3.5 AVX2 RMSNorm routing was rejected because the private scalar Qwen norm stayed faster on this host: Qwen Q4 reruns landed at `27.284460` and `26.969153 tok/s`.
- Qwen3.5 full-attention gate-buffer removal was rejected because direct gating from interleaved `q_gate` slices regressed Qwen Q4 to `27.138908` and `26.602820 tok/s`.
- Qwen3.5 fixed-kernel linear convolution briefly improved a noisy 4-token benchmark sample from `27.200203 tok/s` average to `27.860680 tok/s` average, but was rejected after the more stable 16-token benchmark favored the generic path: `27.680680 tok/s` generic vs `26.034593 tok/s` specialized.
- Qwen3.5 convolution output allocation reuse was rejected because it regressed the Q4 short benchmark from the specialized-path high of `28.209753 tok/s` to `27.501616 tok/s`.
- `fat` LTO was rejected because the Qwen3.5 16-token Q4 benchmark dropped to `26.593991 tok/s`, below the `thin` LTO baseline of `27.680680 tok/s`.
- Public `q4i` integer-activation exposure was rejected because real E2B Q4 inference regressed from `13.261442 tok/s` to `10.645244 tok/s` on the same build.
- Q8/Q5 planned dot dispatcher hoisting was rejected because E2B Q5 dropped to `7.155386 tok/s` versus the prior `7.281891 tok/s`, and E4B Q5 dropped to `3.608818 tok/s` versus `3.715661 tok/s`.
- Hand-written greedy argmax and chunk RMSNorm delegation were rejected because they did not beat the existing iterator/loop layout on real model runs.
- Release `panic = "abort"` and `RUSTFLAGS=-C target-cpu=native` were rejected because both failed to improve E2B Q4 decode throughput on this host.
- Instruction-stream pruning and attention-sparsity flag pass-through were rejected because they reduced E2B Q4 throughput below the env-cache-only path.
- Qwen3.5 linear-attention scratch-buffer reuse was rejected because Qwen Q4 dropped from `27.266739 tok/s` to `25.461866 tok/s`.
- Gemma per-layer projection scale substitutions were rejected after mixed E2B/E4B measurements; the final code keeps the original `powf` math for bit-stable model arithmetic.

## Boundary

These are real local inference executions, not smoke tests or fixtures. They do not prove a global speed record because no same-hardware comparison against llama.cpp, vLLM, SGLang, or other runtimes was executed in this pass.
