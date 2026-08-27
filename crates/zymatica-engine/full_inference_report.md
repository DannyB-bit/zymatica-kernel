# Full Inference Validation Report

- Generated Unix time: `1784182367`
- Overall status: `PASS`

| Model | Engine | New tokens | Exit | Status | Elapsed seconds | Output token count |
| --- | --- | ---: | ---: | --- | ---: | ---: |
| gemma-4-E2B-it | q4 | 16 | 0 | PASS | 148.2 | 17 |
| gemma-4-E4B-it | f32 | 16 | 0 | PASS | 385.4 | 17 |

## Output IDs

### gemma-4-E2B-it

- stdout sha256: `d1cbcc9b53412794c404b4d10336d64acd0bf02a5c3a2436bbd85f2ec856dafd`
- stderr sha256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- output ids: `[2, 236771, 236795, 3677, 236761, 6639, 236769, 3677, 236761, 6639, 236769, 3677, 236761, 6639, 236769, 3677, 236761]`

```text
runtime=zymatica-engine
mode=hf-native-full-inference
engine=q4
selected_engine=q4
model_dir=E:\models\gemma-4-E2B-it
layers=35
hidden_size=1536
prompt_ids=[2]
output_ids=[2, 236771, 236795, 3677, 236761, 6639, 236769, 3677, 236761, 6639, 236769, 3677, 236761, 6639, 236769, 3677, 236761]
elapsed_ms=146289.041
status=ok
```

### gemma-4-E4B-it

- stdout sha256: `5f16517cf41905015908f9db512ba55c62427ffeb9f06730d4a9191c74c592af`
- stderr sha256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- output ids: `[2, 236761, 107, 236769, 236776, 236768, 107, 236769, 236799, 236768, 107, 236769, 236780, 236768, 107, 236769, 236796]`

```text
runtime=zymatica-engine
mode=hf-native-full-inference
engine=f32
selected_engine=f32
model_dir=E:\models\gemma-4-E4B-it
layers=42
hidden_size=2560
prompt_ids=[2]
output_ids=[2, 236761, 107, 236769, 236776, 236768, 107, 236769, 236799, 236768, 107, 236769, 236780, 236768, 107, 236769, 236796]
elapsed_ms=369051.527
status=ok
```

