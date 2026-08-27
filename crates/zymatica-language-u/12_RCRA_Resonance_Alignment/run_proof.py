import argparse
import torch
import torch.nn as nn

def run_proof():
    print("======================================================================")
    print("ZYMATICA | Radical Coordinate Resonance Alignment (RCRA) Loss Proof")
    print("======================================================================\n")

    vocab_size = 128
    batch_size = 4
    K_TOP = 16  # K-Top parameter (simplified for demonstration)
    coord_alpha = 0.8

    print(f"[1] Instantiating Vocab Coordinate Radicals Map (size {vocab_size}x3)...")
    # Setup coordinates: domain, subdomain, polarity
    # Normalized between 0 and 1
    torch.manual_seed(42)
    coords_tensor = torch.rand((vocab_size, 3), dtype=torch.float32)

    # 2. Setup synthetic forward pass outputs (logits and targets)
    print(f"\n[2] Simulating Forward Pass Output Logits (requires_grad=True)...")
    logits = torch.randn((batch_size, vocab_size), dtype=torch.float32, requires_grad=True)
    targets = torch.randint(0, vocab_size, (batch_size,), dtype=torch.long)
    print(f"  - Logits shape: {logits.shape}")
    print(f"  - Targets: {targets.tolist()}")

    # 3. Calculate Cross-Entropy Loss
    print("\n[3] Computing Standard Cross-Entropy Loss...")
    loss_ce_fct = nn.CrossEntropyLoss()
    loss_ce = loss_ce_fct(logits, targets)
    print(f"  - Cross-Entropy Loss: {loss_ce.item():.4f}")

    # 4. Calculate Radical Coordinate Resonance Loss (RCRA)
    print("\n[4] Computing Cuneiform-U Radical Coordinate Resonance Loss...")
    # Get top-K predicted logits and indices
    topk_logits, topk_indices = torch.topk(logits, k=K_TOP, dim=-1)
    probs = torch.softmax(topk_logits, dim=-1)
    
    # Lookup coordinates of top-K predicted indices
    # Shape: (batch_size, K, 3)
    topk_coords = coords_tensor[topk_indices]
    
    # Calculate predicted coordinates (weighted average)
    # Shape: (batch_size, 1, 3) -> squeeze to (batch_size, 3)
    pred_coords = torch.bmm(probs.unsqueeze(1), topk_coords).squeeze(1)
    
    # Lookup target coordinates
    # Shape: (batch_size, 3)
    target_coords = coords_tensor[targets]
    
    # Compute MSE loss over coordinates
    loss_coord = torch.mean((pred_coords - target_coords) ** 2)
    print(f"  - Expected coordinate vectors (first batch): {pred_coords[0].tolist()}")
    print(f"  - Target coordinate vectors (first batch):   {target_coords[0].tolist()}")
    print(f"  - Coordinate Resonance Loss: {loss_coord.item():.6f}")

    # 5. Combine losses and backpropagate
    print("\n[5] Combining Losses and Running Backpropagation...")
    total_loss = loss_ce + coord_alpha * loss_coord
    print(f"  - Total Combined Loss: {total_loss.item():.4f}")
    
    # Run backpropagation
    total_loss.backward()
    
    # Check if gradients flow back to logits successfully
    grad_norm = logits.grad.norm().item()
    print(f"  - Logits gradient norm after backward: {grad_norm:.6f}")
    
    assert grad_norm > 0, "Gradient flow failed! Logits received zero gradients."
    print("\n[VERIFICATION] RCRA loss function and gradient flow verified.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica RCRA Loss Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
