#!/usr/bin/env python3
import sys
import torch
import torch.nn as nn
from transformers.models.gemma4.configuration_gemma4 import Gemma4TextConfig
from transformers.models.gemma4.modeling_gemma4 import Gemma4ForCausalLM

def main():
    if len(sys.argv) < 2:
        print("Usage: generate_tiny_hf_model.py <output_dir>")
        sys.exit(1)
    
    output_dir = sys.argv[1]
    
    # Native Gemma 4 configuration with 1 layer (matching basic test structure)
    config = Gemma4TextConfig(
        vocab_size=1000,
        hidden_size=256,
        intermediate_size=512,
        num_hidden_layers=1,
        num_attention_heads=1,
        num_key_value_heads=1,
        head_dim=256,
        rms_norm_eps=1e-6,
        max_position_embeddings=128,
        vocab_size_per_layer_input=0,
        hidden_size_per_layer_input=0,
        num_kv_shared_layers=0,
        global_head_dim=256,
        layer_types=["full_attention"],
        rope_parameters={
            "full_attention": {
                "partial_rotary_factor": 1.0,
                "rope_theta": 10000.0,
                "rope_type": "default"
            }
        }
    )
    
    # Initialize model with deterministic weights
    torch.manual_seed(42)
    model = Gemma4ForCausalLM(config)
    model.train()
    
    # Target sequence of 17 non-repeating tokens to guarantee deep semantic correctness
    target_sequence = [2, 331, 110, 199, 250, 88, 712, 5, 999, 12, 300, 45, 800, 150, 600, 22, 555]
    
    input_ids = torch.tensor([target_sequence[:-1]], dtype=torch.long)
    targets = torch.tensor([target_sequence[1:]], dtype=torch.long)
    
    optimizer = torch.optim.AdamW(model.parameters(), lr=1e-2)
    criterion = nn.CrossEntropyLoss()
    
    print("Training model to predict varied tokens...")
    for epoch in range(120):
        optimizer.zero_grad()
        outputs = model(input_ids)
        logits = outputs.logits
        loss = criterion(logits.view(-1, 1000), targets.view(-1))
        loss.backward()
        optimizer.step()
        
    print(f"Training complete. Loss: {loss.item():.6f}")
    
    # Convert to bfloat16 to match target deployment and verify it remains stable
    model.to(torch.bfloat16)
    
    # Verify greedy generation in bfloat16
    model.eval()
    gen_input = torch.tensor([[2]], dtype=torch.long)
    generated = [2]
    for _ in range(16):
        with torch.no_grad():
            outputs = model(gen_input)
            next_token = torch.argmax(outputs.logits[0, -1, :]).item()
            generated.append(next_token)
            gen_input = torch.cat([gen_input, torch.tensor([[next_token]])], dim=-1)
            
    print("Expected: ", target_sequence)
    print("Generated (BF16):", generated)
    
    # Save the model
    model.save_pretrained(output_dir, safe_serialization=True)
    
    # Overwrite config.json manually to ensure Zymatica loader constraints are met
    import json
    config_path = f"{output_dir}/config.json"
    with open(config_path, "r") as f:
        cfg_data = json.load(f)
        
    cfg_data["torch_dtype"] = "bfloat16"
    cfg_data["model_type"] = "gemma4_text"
    cfg_data["global_head_dim"] = 256
    cfg_data["hidden_size_per_layer_input"] = 0
    cfg_data["vocab_size_per_layer_input"] = 0
    cfg_data["layer_types"] = ["full_attention"]
    cfg_data["rope_parameters"] = {
        "full_attention": {
            "partial_rotary_factor": 1.0,
            "rope_theta": 10000.0,
            "rope_type": "default"
        }
    }
    
    with open(config_path, "w") as f:
        json.dump(cfg_data, f, indent=2)
        
    print(f"Trained Gemma 4 model saved to {output_dir}")

if __name__ == "__main__":
    main()
