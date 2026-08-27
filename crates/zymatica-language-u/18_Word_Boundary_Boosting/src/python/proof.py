import argparse
import torch
import torch.nn.functional as F

# Mock vocabulary database
MOCK_VOCAB = {
    0: "Ġthe",       # Function word with boundary
    1: "Ġis",        # Function word with boundary
    2: "Ġgateway",   # Content word with boundary
    3: "Ġreset",     # Content word with boundary
    4: "apple",      # Content word without boundary
    5: "ing",        # Fragment
    6: "tion",       # Fragment
    7: "Ġa"          # Short word with boundary
}

_FUNC_WORDS = {"the", "is", "a", "an", "of", "to", "in", "for"}
WBB_WORD_BOOST = 3.5
WBB_FUNC_BOOST = 1.5
WBB_FRAG_BOOST = 1.0

def build_wbb_boost_vector(vocab_size):
    """Calculates the static WBB boost vector over the vocabulary."""
    wbb = torch.zeros(vocab_size, dtype=torch.float32)
    for i in range(vocab_size):
        t = MOCK_VOCAB[i]
        # Check boundary prefix (SentencePiece space symbol or Qwen 'Ġ')
        has_boundary = t.startswith("Ġ") or t.startswith(" ") or t.startswith("\u2581")
        clean_word = t.replace("Ġ", "").replace(" ", "").replace("\u2581", "").lower()
        
        if not clean_word:
            continue
            
        if has_boundary:
            if clean_word in _FUNC_WORDS:
                wbb[i] = WBB_FUNC_BOOST
            elif len(clean_word) >= 2:
                wbb[i] = WBB_WORD_BOOST
        else:
            if len(clean_word) >= 3:
                wbb[i] = WBB_FRAG_BOOST
    return wbb

def sample_next_token(logits, temperature=0.7, top_k=40, top_p=0.90):
    """Sampler with top-p/top-k from test_sampling.py."""
    if temperature <= 0:
        return torch.argmax(logits).item()
    logits = logits / temperature
    if top_k > 0:
        kth_val = torch.topk(logits, min(top_k, logits.size(-1))).values[-1]
        logits = logits.masked_fill(logits < kth_val, float('-inf'))
    if top_p < 1.0:
        sorted_logits, sorted_idx = torch.sort(logits, descending=True)
        cum_probs = torch.cumsum(F.softmax(sorted_logits, dim=-1), dim=-1)
        shifted_cum = torch.cat([torch.zeros(1, device=cum_probs.device), cum_probs[:-1]])
        sorted_logits[shifted_cum > top_p] = float('-inf')
        logits = torch.zeros_like(logits).scatter_(0, sorted_idx, sorted_logits)
    probs = F.softmax(logits, dim=-1)
    if torch.isnan(probs).any() or probs.sum() == 0:
        return torch.argmax(logits).item()
    return torch.multinomial(probs, num_samples=1).item()

def run_proof():
    print("======================================================================")
    print("ZYMATICA | Word-Boundary Boosting (WBB) Logits Steering Proof")
    print("======================================================================\n")

    vocab_size = len(MOCK_VOCAB)
    wbb = build_wbb_boost_vector(vocab_size)

    print("[1] MOCK Vocabulary & Calculated WBB Boost Factors:")
    for i in range(vocab_size):
        token = MOCK_VOCAB[i]
        print(f"  Token {i}: '{token.replace('Ġ', '_'):12s}' -> WBB Boost: {wbb[i].item():.1f}")

    # Simulate flat, uncertain logits output from a compressed model
    print("\n[2] Simulating Flat/Uncertain Logits (Unsteered Outputs)...")
    torch.manual_seed(42)
    # Set all base logits close to zero to represent high entropy/uncertainty
    logits = torch.zeros(vocab_size)
    print(f"  - Initial Logits: {logits.tolist()}")

    # Output probabilities before boost
    probs_raw = F.softmax(logits, dim=-1)
    print(f"  - Raw Probabilities: {[round(p, 4) for p in probs_raw.tolist()]}")

    # 3. Apply WBB
    print("\n[3] Applying Word-Boundary Boost (logits_boosted = logits + wbb)...")
    logits_boosted = logits + wbb
    probs_boosted = F.softmax(logits_boosted, dim=-1)
    
    print(f"  - Boosted Logits: {logits_boosted.tolist()}")
    print(f"  - Boosted Probabilities:")
    for i in range(vocab_size):
        token = MOCK_VOCAB[i]
        print(f"    * '{token.replace('Ġ', '_'):12s}': {probs_raw[i].item()*100:5.2f}% -> {probs_boosted[i].item()*100:5.2f}%")

    # 4. Run sampling simulation
    print("\n[4] Running 1000 Sampling Iterations to Measure Selection Bias...")
    raw_samples = [sample_next_token(logits) for _ in range(1000)]
    boosted_samples = [sample_next_token(logits_boosted) for _ in range(1000)]
    
    # Calculate boundary selection rates
    boundary_ids = [i for i in range(vocab_size) if MOCK_VOCAB[i].startswith("Ġ")]
    
    raw_boundary_rate = sum(1 for s in raw_samples if s in boundary_ids) / 1000.0 * 100
    boosted_boundary_rate = sum(1 for s in boosted_samples if s in boundary_ids) / 1000.0 * 100
    
    print(f"  - Word Boundary Selection Rate (Raw):     {raw_boundary_rate:.2f}%")
    print(f"  - Word Boundary Selection Rate (Boosted): {boosted_boundary_rate:.2f}%")
    
    assert boosted_boundary_rate > raw_boundary_rate, "WBB failed to bias towards boundaries!"
    print("\n[VERIFICATION] Word-Boundary Boosting verified successfully.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica WBB Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
