# Agent Runtime Real Benchmarks

Date: 2026-07-19

These are real release-binary executions against the local full Gemma E2B checkpoint and existing q5 cache. They are not tiny fixtures, seeded models, or mock paths.

Model:

- `E:\models\gemma-4-E2B-it`
- tokenizer: `E:\models\gemma-4-E2B-it\tokenizer.json`
- cache: `E:\models\gemma-4-E2B-it\.zymatica-cache-q5`
- layers: 35
- hidden size: 1536
- engine: q5

## Standard Real Inference Benchmark

Command:

```powershell
target\release\zymatica-engine.exe benchmark-inference --model-dir E:\models\gemma-4-E2B-it --prompt-ids 2 --new-tokens 1 --engine q5 --q8-cache-dir E:\models\gemma-4-E2B-it\.zymatica-cache-q5
```

Result:

| Metric | Value |
| --- | ---: |
| output IDs | `[2, 236761]` |
| load ms | 68,124.279 |
| prefill ms | 6,983.390 |
| TTFT ms | 6,985.573 |
| cold TTFT ms | 75,109.852 |
| generation ms | 6,985.573 |
| end-to-end tok/s | 0.143152 |
| completion NLL | 1.501063 |
| completion perplexity | 4.486457 |

The dominant cost is still full q5 model/cache load. Warm request scheduling and mmap residency remain the main path for production throughput.

## Cache-To-Cache Packet Improvement

The original agent cache-to-cache path reused the page-granular KV swap layout. That is correct, but it serializes the unused tail of the final KV page. The new `compact-token-kv-v2` packet writes only resident token KV cells while preserving exact SHA-256 integrity and shape validation.

Baseline command before compact packets:

```powershell
target\release\zymatica-engine.exe agent-cache-to-cache-run --model-dir E:\models\gemma-4-E2B-it --tokenizer E:\models\gemma-4-E2B-it\tokenizer.json --prompt "Zymatica cache transfer field test" --new-tokens 2 --engine q5 --q8-cache-dir E:\models\gemma-4-E2B-it\.zymatica-cache-q5
```

Improved command after compact packets:

```powershell
target\release\zymatica-engine.exe agent-cache-to-cache-run --model-dir E:\models\gemma-4-E2B-it --tokenizer E:\models\gemma-4-E2B-it\tokenizer.json --prompt "Zymatica cache transfer field test" --new-tokens 2 --engine q5 --q8-cache-dir E:\models\gemma-4-E2B-it\.zymatica-cache-q5
```

Result:

| Run | Packet Format | Page Packet Bytes | Sent Packet Bytes | Reduction | Elapsed ms | Output |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Baseline | page KV packet | 688,736 | 688,736 | 0.000% | 286,031.490 | `Zymatica cache transfer field test test field` |
| Improved | compact-token-kv-v2 | 688,736 | 602,712 | 12.490% | 297,485.305 | `Zymatica cache transfer field test test field` |

The continuation output stayed identical after import into a fresh paged KV cache:

```text
output_ids=[236953, 1177, 61716, 15612, 4921, 2135, 1594, 1594, 2135]
output_text=Zymatica cache transfer field test test field
packet_sha256=ad47de09c63742c52cb3f94441a749ae1173456bc3023b35d578336604104167
```

The benchmark prompt has 7 prompt tokens and a page size of 8, so compacting removes one unused page slot. Larger savings appear on shorter sequences or larger page sizes; smaller savings appear when sequence lengths are already close to a page boundary.

## Schema-Masked Real JSON Generation

Command:

```powershell
target\release\zymatica-engine.exe agent-json-run --model-dir E:\models\gemma-4-E2B-it --tokenizer E:\models\gemma-4-E2B-it\tokenizer.json --prompt "Return JSON only: " --fields answer --max-new-tokens 48 --min-string-chars 1 --max-string-chars 4 --engine q5 --q8-cache-dir E:\models\gemma-4-E2B-it\.zymatica-cache-q5
```

Result:

```text
generated_tokens=13
masked_steps=13
json_text={"answer":"a_n_"}
json_parsed={"answer":"a_n_"}
elapsed_ms=359,092.528
status=ok
```

This proves the schema mask runs inside real token generation and produces parseable JSON without post-hoc repair. The remaining performance issue is that every masked generation step still scans the full vocabulary; a trie or token-prefix cache is the next improvement.

## Immediate Next Improvements

1. Keep a long-lived model worker warm for agent benchmarks so load time is not paid per command.
2. Add token-prefix indexes for schema masks to avoid full-vocabulary scans each masked step.
3. Add optional zstd compression over compact KV packets after the compact uncompressed format is stable.
4. Add packet delta mode for branching agents so child branches transmit only KV pages/tokens beyond their shared prefix.
5. Extend compact cache packets to Qwen3.5 native mixed caches after Qwen quantized/cache paths land.
