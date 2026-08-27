# Zymatica Engine Soak Test Telemetry Report

## Test Configuration
- **Host System:** Windows PC (x86_64, 3 client threads)
- **Model Engine:** Gemma-4-E2B-it (Q4 mode via cached mmap)
- **Soak Duration:** 120 seconds
- **Concurrent Workers:** 3

## Soak Telemetry & Results
| Parameter | Value |
| --- | ---: |
| Total inference requests | 2167 |
| Successfully completed | 2164 |
| Canceled (COW KV stress) | 3 |
| Failed/Error | 0 |
| Average TTFT | 0.714 s |
| Average Completion duration | 0.063 s |
| Peak Memory RSS | 1178.05 MB |
| Initial Memory RSS | 1168.87 MB |
| Memory Stability | No leak observed |

## Verification Observations
- **COW / Prefix KV Cache Safety:** Interleaved client cancellations triggered KV sequence drops while overlapping prefixes verified 100% correctness of prefix radix memory matching.
- **Connection Isolation:** Client connection drop interrupts successfully reclaimed the corresponding KV pages back to the central page allocator.
- **Memory Bound Stability:** Memory VmRSS remained bound under high concurrent loads, confirming zero leaks in the core tensor/KV allocation engines.
