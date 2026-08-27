# Zymatica Engine: Raspberry Pi 4 Physical Hardware Performance Report

This report records the reported physical performance characteristics of
Zymatica Engine running `Gemma-4-E2B-it` on a Raspberry Pi 4 Model B.

Hardware target:

- Raspberry Pi 4 Model B
- Broadcom BCM2711
- 4x Cortex-A72 cores at 1.5 GHz
- 8 GB RAM
- Active fan cooling unless otherwise noted

This file is ASCII-only so it renders cleanly in Windows terminals, CI logs, and
GitHub evidence bundles.

## Core generation performance

The ARM Cortex-A72 does not provide native INT8 dot-product instructions
(`sdot`/`udot`). Zymatica therefore uses NEON vector lanes and low-precision
unpacking into f32 accumulation paths.

| Quantization level | Reported model size | Reported TTFT | Reported throughput |
| --- | ---: | ---: | ---: |
| Q8 SVD/row quantized | 2.10 GB | 820 ms | 1.85 tokens/sec |
| Q5 scaled | 1.35 GB | 610 ms | 2.74 tokens/sec |
| Q4 nibble | 1.10 GB | 495 ms | 3.28 tokens/sec |

## Memory footprint

| Mode | Reported peak memory use |
| --- | ---: |
| System baseline, Raspbian Lite/headless | 145 MB |
| Q8 mode | 2.25 GB |
| Q5 mode | 1.52 GB |
| Q4 mode | 1.28 GB |

The reported memory behavior is consistent with memory-mapped weights plus a
bounded activation/KV working set. Raw RSS logs should be included in the final
field evidence bundle for audit-grade verification.

## Thermal diagnostics and throttling behavior

### Active cooling

- Idle temperature: 38C
- Peak temperature: 54C after warm-up
- Reported throttling state: `0x0`
- Result: no throttling observed

### Passive/no cooling

- Idle temperature: 48C
- Peak temperature: 82C within approximately 3 minutes
- Reported throttling state: `0x20000`
- Result: ARM frequency capping observed, with Q8 throughput dropping to roughly
  1.1 tokens/sec

## Long-run stability

Reported long-run test:

- Context window: 4,096 tokens
- Duration: 30 minutes
- Output determinism: identical token IDs for identical seeds
- KV cache behavior: stable block reallocation
- Memory leak status: reported as zero leaked bytes

For final audit, preserve the raw command output and telemetry:

- `vcgencmd measure_temp`
- `vcgencmd get_throttled`
- `/proc/meminfo`
- process RSS samples
- CPU frequency samples
- generated token IDs
- benchmark command lines
- binary/model/quant artifact SHA256 hashes
