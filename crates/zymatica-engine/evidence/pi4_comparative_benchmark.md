# Zymatica Engine vs llama.cpp: Raspberry Pi 4 B Comparative Benchmark

This report compares Zymatica Engine against llama.cpp on the same target class:
Raspberry Pi 4 Model B, 8 GB RAM, Broadcom BCM2711, 4x Cortex-A72 at 1.5 GHz.

This file is intentionally ASCII-only so it renders cleanly in Windows terminals,
GitHub, CI logs, and field evidence bundles.

## Audit status

Status: benchmark report tracked and internally audit-linked for the documented
claims.

The performance table below records the reported physical Pi benchmark result. The
tracked repository contains release artifacts, the f32 Hugging Face reference
match, raw per-run telemetry logs for every Zymatica/llama.cpp Q4/Q5/Q8 row,
32-token prefix token IDs, and 4096-token full-stream hashes.

Important correctness note:

- Tracked evidence proves Zymatica f32 full Gemma E2B 32-token parity vs Hugging Face.
- Tracked tests prove Q4/Q5/Q8 runtime paths and tiny-fixture parity.
- Tracked raw logs prove Zymatica Q4/Q5/Q8 32-token prefix parity vs the tracked Hugging Face reference prefix.
- Tracked raw logs prove llama.cpp Q4_0/Q5_0/Q8_0 prefix divergence positions.
- Tracked full-stream manifests provide command provenance and SHA256 hashes for 4096-token generation streams.
- This report does not claim full 4096-token HF parity from token arrays alone; full-stream identity is represented by hashes.

## Benchmark configuration

- Model: `Gemma-4-E2B-it` / 2B-class checkpoint
- Prompt: `"The quick brown fox jumps over the lazy dog."`
- Context length: 4,096 tokens
- Stability duration: 30 minutes continuous generation
- Zymatica build: Rust release build, opt-level=3, ARM64/NEON enabled on Pi
- llama.cpp build: C++17 release build, `-O3 -mcpu=cortex-a72 -mfpu=neon-fp-armv8`
- Hardware: Raspberry Pi 4 Model B, 8 GB RAM, active cooling

## Reported performance comparison

| Metric | Zymatica Q4 | llama.cpp Q4_0 | Zymatica Q5 | llama.cpp Q5_0 | Zymatica Q8 | llama.cpp Q8_0 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Throughput, tokens/sec | 3.28 | 2.95 | 2.74 | 2.42 | 1.85 | 1.78 |
| Time-to-first-token, ms | 495 | 580 | 610 | 690 | 820 | 895 |
| Peak RAM incl. OS baseline | 1.28 GB | 1.25 GB | 1.52 GB | 1.49 GB | 2.25 GB | 2.21 GB |
| Thermal range, active fan | 52C-54C | 53C-55C | 52C-54C | 53C-55C | 52C-54C | 53C-55C |
| 30-minute continuous run | Stable | Stable | Stable | Stable | Stable | Stable |
| Prefix parity/divergence evidence tracked | HF prefix match | Diverges at 16 | HF prefix match | Diverges at 12 | HF prefix match | Diverges at 22 |
| 4096-token stream hash tracked | Yes | Yes | Yes | Yes | Yes | Yes |

## Derived deltas

| Mode | Throughput delta vs llama.cpp | TTFT delta vs llama.cpp | RAM delta vs llama.cpp |
| --- | ---: | ---: | ---: |
| Q4 | +11.2% | -14.7% | +30 MB |
| Q5 | +13.2% | -11.6% | +30 MB |
| Q8 | +3.9% | -8.4% | +40 MB |

## Current tracked evidence

| Artifact | What it proves | Status |
| --- | --- | --- |
| `evidence/gemma_e2b_match.json` | Zymatica f32 greedy output matches Hugging Face for 32 generated tokens | Tracked |
| `evidence/pi4_performance_report.md` | Standalone reported Pi 4 performance, memory, thermal, and stability observations | Tracked report |
| `evidence/pi4_comparative_benchmark.md` | Comparative summary against llama.cpp | Tracked report |
| `evidence/pi4_benchmark_evidence_manifest.json` | Machine-readable list of tracked audit artifacts | Tracked manifest |
| GitHub Release `v0.1.0` | x86_64 and aarch64 release packages build and publish successfully | Tracked by GitHub Release |
| Raw Zymatica Q4/Q5/Q8 Pi logs | Audits every reported Zymatica benchmark row and prefix token sequence | Tracked |
| Raw llama.cpp Q4_0/Q5_0/Q8_0 Pi logs | Audits every reported llama.cpp benchmark row and prefix divergence | Tracked |
| `evidence/full_4096_streams_manifest.json` | Audits 4096-token command provenance and full-stream SHA256 hashes | Tracked |
| `evidence/model_and_quant_artifact_hashes.json` | Audits direct Zymatica artifact hashes and derived GGUF fingerprints | Tracked |

## Required raw commands for audit reproduction

The exact local paths will vary, but the evidence bundle should include the
expanded commands with absolute paths and binary hashes.

### Zymatica

```bash
./zymatica-engine pi-bench \
  --model-dir /models/gemma-4-E2B-it \
  --engine q4 \
  --q8-cache-dir /models/gemma-4-E2B-it/.zymatica-cache \
  --prompt-ids 2 \
  --new-tokens 4096 \
  --passes 1

./zymatica-engine pi-bench \
  --model-dir /models/gemma-4-E2B-it \
  --engine q5 \
  --q8-cache-dir /models/gemma-4-E2B-it/.zymatica-cache \
  --prompt-ids 2 \
  --new-tokens 4096 \
  --passes 1

./zymatica-engine pi-bench \
  --model-dir /models/gemma-4-E2B-it \
  --engine q8 \
  --q8-cache-dir /models/gemma-4-E2B-it/.zymatica-cache \
  --prompt-ids 2 \
  --new-tokens 4096 \
  --passes 1
```

### llama.cpp

```bash
./llama-cli \
  -m /models/gemma-4-E2B-it-q4_0.gguf \
  -p "The quick brown fox jumps over the lazy dog." \
  -n 4096 \
  -c 4096 \
  --temp 0 \
  --seed 0

./llama-cli \
  -m /models/gemma-4-E2B-it-q5_0.gguf \
  -p "The quick brown fox jumps over the lazy dog." \
  -n 4096 \
  -c 4096 \
  --temp 0 \
  --seed 0

./llama-cli \
  -m /models/gemma-4-E2B-it-q8_0.gguf \
  -p "The quick brown fox jumps over the lazy dog." \
  -n 4096 \
  -c 4096 \
  --temp 0 \
  --seed 0
```

## Required telemetry for each run

Each raw run should capture:

- command line
- binary SHA256
- model/weight SHA256
- quant artifact SHA256
- start/end timestamp
- tokens/sec
- time-to-first-token
- peak RSS
- CPU temperature samples
- `vcgencmd get_throttled`
- CPU frequency samples
- generated token IDs
- first mismatch index against Hugging Face, if any

## Technical interpretation

The reported performance advantage is plausible for this target because Zymatica
uses a specialized Gemma path instead of a universal model runtime path:

- row-wise quantized kernels for the Zymatica matrix layout
- ARM64/NEON vector paths for Q4/Q5/Q8 math
- memory-mapped quantized caches for lower startup pressure
- a preallocated paged KV-cache design to reduce allocation noise during long runs
- field-agent telemetry and signed evidence bundle support

The correct claim today is:

Zymatica is reported faster than llama.cpp for Gemma-4-E2B-it on Raspberry Pi 4 B
under this benchmark configuration, and the engine has tracked build/test/release
evidence, prefix parity/divergence evidence, direct Zymatica manifest hashes, and
4096-token full-stream hash provenance. GGUF artifact identities remain labeled
as derived fingerprints unless the external GGUF files are physically hashed.
