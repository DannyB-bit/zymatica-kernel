# Zero-Materialization Improvement Plan

The current `--in-memory` path proves zero disk materialization: the engine accepts the compressed `.ufomodel.zip`, verifies it, keeps the decoded files in heap-backed `ByteStorage::Memory`, and runs inference without creating the safetensors cache directory.

The main remaining cost is RAM materialization. Lossless capsules still expand full safetensors weights into memory before the current f32/q8/q5/q4 loaders consume them. Current source-resident payloads are 9,803 MiB for E2B and 15,283 MiB for E4B before the selected runtime's own matrices and KV cache are counted. These are the highest-impact improvements.

## 1. Executable Quantized UFO Capsules

Initial v2 support is implemented: `scripts/package_quantized_capsule_v2.py` creates `ufo-v2` / `quantized` capsules with Zymatica-native `.zq8`, `.zq5`, or `.zq4` tensors directly instead of full safetensors for large matrices. The engine can load these pre-quantized tensors from memory and skip load-time quantization for those members.

Integrity status: v2 manifests now require direct SHA-256s for every member. Empty hashes are rejected by the Rust capsule loader before extraction or in-memory execution.

Expected impact:

- much lower peak RAM in `--in-memory`
- much faster cold start
- smaller capsules than lossless safetensors ZIPs
- reproducible quality gates per quant mode

Remaining correctness requirement: this is not lossless unless the artifact passes logit/token parity suites. Label it `quantized-calibrated`, not `lossless`.

## 2. Chunked Tensor Capsule Index

Replace whole-file ZIP members for weights with a tensor/chunk table:

- tensor name
- dtype and shape
- chunk offsets
- chunk checksums
- compression method
- optional quantization metadata

This lets the engine fetch only the tensor or row block it needs instead of inflating a full safetensors shard.

## 3. Streaming Layer Loader

For quantized modes, consume one tensor at a time from the capsule, quantize it into the resident runtime matrix, then release the decompressed source bytes immediately. This removes the current need to keep the full lossless safetensors payload alive while the quantized model exists.

## 4. Calibration-Aware Default Capsules

Use AWQ/GPTQ-style activation calibration to produce the default edge capsule:

- q5 for balanced desktop/edge
- q4 for Raspberry Pi memory pressure
- q8 for reference-grade parity

Publish the calibration prompts, tensor scales, error histograms, token parity, perplexity deltas, and exact artifact hashes.

## 5. Layer-Fused CPU Kernels

Move from per-projection matvecs to fused layer kernels:

- fused Q/K/V projection with GQA layout awareness
- fused RMSNorm + matvec input scaling
- fused gate/up MLP pass
- Q8 activation path by default where parity allows it

This attacks the current CPU memory bandwidth bottleneck.

## 6. GPU Backend Only Behind Parity Gates

The GPU path should stay optional until it passes the same deterministic parity harness:

- CPU f32 reference
- quantized CPU reference
- GPU output token equality
- logit tolerance bands per layer
- TTFT/TPS/RSS/VRAM telemetry

The right target is not just "GPU exists"; it is "GPU is faster without losing certified behavior."

## 7. Evidence Gate Before Release Claims

Every model capsule release should carry:

- capsule SHA-256
- source model file hashes
- generated token arrays
- TTFT/TPS/perplexity telemetry
- peak RSS
- command logs
- HF parity report at 512, 1024, and 3264 tokens
- strict external artifact hashes where GGUF comparisons are claimed

Without this, the system can be impressive but not yet production-certifiable.
