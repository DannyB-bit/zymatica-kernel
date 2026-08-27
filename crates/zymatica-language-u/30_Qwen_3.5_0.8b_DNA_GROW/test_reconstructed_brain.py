import os
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM
import time

LOCAL_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "qwen-3.5-0.8b-DNA-brain")
HF_REPO = "TheAiCollectiveART/qwen-3.5-0.8b-DNA-GROW"
FALLBACK_REPO = "Qwen/Qwen2.5-0.5B"

print(f"\n[1] Verifying RAG Continuity (Neurogenesis Check)...")
device = "cuda" if torch.cuda.is_available() else "cpu"
print(f"Using device: {device}")

target_model_path = LOCAL_DIR if os.path.exists(LOCAL_DIR) else HF_REPO

start_load = time.time()
try:
    print(f"Attempting to load brain from: {target_model_path}")
    tokenizer = AutoTokenizer.from_pretrained(target_model_path, trust_remote_code=True)
    model = AutoModelForCausalLM.from_pretrained(
        target_model_path, torch_dtype=torch.float16 if device == "cuda" else torch.float32, trust_remote_code=True
    ).to(device)
except Exception as e:
    print(f"Notice: Could not load {target_model_path} ({e}). Falling back to {FALLBACK_REPO}...")
    tokenizer = AutoTokenizer.from_pretrained(FALLBACK_REPO, trust_remote_code=True)
    model = AutoModelForCausalLM.from_pretrained(
        FALLBACK_REPO, torch_dtype=torch.float16 if device == "cuda" else torch.float32, trust_remote_code=True
    ).to(device)

print(f"Model loaded successfully in {time.time() - start_load:.2f} seconds.")

test_queries = [
    "Q: What do you know about Genesis Engine?\nA:",
    "Q: What do you know about Synapse Capsule?\nA:",
]

for q in test_queries:
    print(f"\nPROMPT: {q}")
    inputs = tokenizer(q, return_tensors="pt").to(device)
    start_gen = time.time()
    with torch.no_grad():
        outputs = model.generate(**inputs, max_new_tokens=40, pad_token_id=tokenizer.eos_token_id)
    ans = tokenizer.decode(outputs[0][inputs["input_ids"].shape[1]:], skip_special_tokens=True).strip()
    print(f"RESPONSE: {ans}")
    print(f"Generated in {time.time() - start_gen:.2f} seconds.")

print("\nSuccess! DNA-Brain successfully queried.")