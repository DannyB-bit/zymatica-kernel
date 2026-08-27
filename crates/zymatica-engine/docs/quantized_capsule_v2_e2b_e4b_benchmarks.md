# Quantized UFO Capsule v2 Benchmarks: Gemma E2B and E4B

Date: 2026-07-18

These results are real release-build executions of full Gemma E2B/E4B artifacts packaged as executable UFO v2 q5 capsules and run by Zymatica Engine with `--in-memory`.

Important fidelity boundary: q5 v2 is an executable quantized capsule path. It exact-matches the Zymatica q5 token path for the tested prompt, but it is not a mathematically lossless f32/HF artifact. Full HF/f32 long-token parity remains a separate certification target.

## Artifacts

| Model | Source artifact | Source size | Lossless capsule | q5 v2 capsule | q5 v2 SHA-256 |
| --- | --- | ---: | ---: | ---: | --- |
| E2B | `I:\cache\zymatica-artifacts\extracted\gemma-4-E2B-it-full-lossless` | 9.603 GiB | 7.282 GiB | 5.012 GiB | `67cf1fed86d85bd70e9412b30ce02a22ad2635806ee8043f63c29cbaf38c75ee` |
| E4B | `I:\cache\zymatica-artifacts\models\gemma-4-E4B-it-full` | 14.924 GiB | 11.339 GiB | 7.099 GiB | `c4de644903554bfb3b9b7ee13cd5a8486bf17b9be90db42be5929c017249eed5` |

## Integrity Validation

Command shape:

```powershell
target\release\zymatica-engine.exe verify-capsule --capsule <capsule>
```

| Model | ZIP test | Manifest entries | q5 tensor shards | Safetensors remainder | Direct member hashes |
| --- | --- | ---: | ---: | ---: | --- |
| E2B | ok | 481 | 478 | 1 | all present, 64 hex chars |
| E4B | ok | 544 | 541 | 1 | all present, 64 hex chars |

Strict verifier output:

| Model | Format | Mode | Source bytes | Stored payload bytes | ZIP entries | Raw files | UFO files | Direct SHA-256 count | Status |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| E2B | `ufo-v2` | `quantized` | 6,845,987,362 | 6,845,987,362 | 482 | 481 | 0 | 481 | ok |
| E4B | `ufo-v2` | `quantized` | 9,292,719,455 | 9,292,719,455 | 545 | 544 | 0 | 544 | ok |

## In-Memory Run Results

Command shape:

```powershell
target\release\zymatica-engine.exe run-capsule --capsule <capsule> --prompt-ids 2 --new-tokens 8 --engine q5 --in-memory
```

| Model | Layers | Hidden | Source resident | Output IDs | Elapsed |
| --- | ---: | ---: | ---: | --- | ---: |
| E2B | 35 | 1536 | 6529 MB | `[2, 236761, 108, 1018, 8291, 659, 496, 2321, 3835]` | 3276.532 ms |
| E4B | 42 | 2560 | 8863 MB | `[2, 236761, 108, 236829, 808, 808, 108, 1018, 818]` | 6234.311 ms |

Both runs reported:

```text
capsule_in_memory=true
capsule_disk_cache=disabled
capsule_materialization=memory-only
selected_engine=q5
status=ok
```

## Benchmark Telemetry

Command shape:

```powershell
target\release\zymatica-engine.exe benchmark-capsule --capsule <capsule> --prompt-ids 2 --new-tokens 8 --engine q5 --in-memory
```

| Model | TTFT | Cold TTFT | Decode TPS | End-to-end TPS | Completion NLL | Completion perplexity |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| E2B | 331.084 ms | 678.736 ms | 3.100481 | 3.072099 | 5.076033 | 1.886087 |
| E4B | 661.527 ms | 1093.594 ms | 1.556797 | 1.546389 | 8.323681 | 2.830519 |

Raw logs:

- `I:\cache\zymatica-artifacts\logs\package-e2b-q5-v2.log`
- `I:\cache\zymatica-artifacts\logs\final-verify-e2b-q5-v2-capsule.stdout.log`
- `I:\cache\zymatica-artifacts\logs\final-run-e2b-q5-v2-in-memory.stdout.log`
- `I:\cache\zymatica-artifacts\logs\final-bench-e2b-q5-v2-in-memory.stdout.log`
- `I:\cache\zymatica-artifacts\logs\package-e4b-q5-v2.log`
- `I:\cache\zymatica-artifacts\logs\final-verify-e4b-q5-v2-capsule.stdout.log`
- `I:\cache\zymatica-artifacts\logs\final-run-e4b-q5-v2-in-memory.stdout.log`
- `I:\cache\zymatica-artifacts\logs\final-bench-e4b-q5-v2-in-memory.stdout.log`

## Loader Fix Required By E4B

The first E4B execution failed honestly before inference:

```text
Error: loading quantized Q5 in-memory
Caused by:
    matrix size exceeds budget: 671088640 elements
```

That tensor scale is expected for the E4B embedding/LM-head shape `262144 x 2560`. The loader now uses a named `MAX_QUANTIZED_MATRIX_ELEMENTS` guard of `1_000_000_000`, with a regression test proving the E4B-sized q5 shape is accepted while larger unbounded headers are still rejected.
