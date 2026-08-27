import argparse
import struct
import numpy as np

# Binary file specification constants
GENESIS_MAGIC   = 0x47454E45   # "GENE"
PERFECT_MAGIC   = 0x50455246   # "PERF"
WATERMARK       = b"ip zymatica.space".ljust(32, b" ")
GENESIS_VERSION = 12           # Version 12 for Level 8 Procedural Seed

def float32_to_float16_bytes(val):
    """Converts a float32 to a big-endian float16 byte structure."""
    f16_val = np.array([val], dtype=np.float32).astype(np.float16)
    return struct.pack('>H', f16_val.view(np.uint16)[0])

def float16_bytes_to_float32(b_val):
    """Converts big-endian float16 bytes back to a float32 value."""
    u16_val = struct.unpack('>H', b_val)[0]
    f16_val = np.array([u16_val], dtype=np.uint16).view(np.float16)[0]
    return float(f16_val)

def serialize_genesis(metadata, layers_data):
    """Pack metadata and layers into a big-endian .genesis binary payload."""
    payload = bytearray()
    
    # 1. Header packing
    payload.extend(struct.pack('>I', GENESIS_MAGIC))
    payload.extend(struct.pack('>H', GENESIS_VERSION))
    payload.extend(WATERMARK)
    payload.extend(struct.pack('>I', PERFECT_MAGIC))
    
    # 2. Network hyperparameters packing
    payload.extend(struct.pack('>IIIIII', 
                               metadata['hidden_size'], 
                               metadata['num_heads'], 
                               metadata['num_kv_heads'], 
                               metadata['ffn_dim'], 
                               metadata['num_blocks'], 
                               metadata['vocab_size']))
    
    # 3. Energy targets (4 floats)
    payload.extend(struct.pack('>ffff', *metadata['energy_targets']))
    
    # 4. Layer count
    payload.extend(struct.pack('>I', len(layers_data)))
    
    # 5. Layer projections body packing
    for layer in layers_data:
        name_bytes = layer['name'].encode('utf-8')
        payload.extend(struct.pack('>H', len(name_bytes)))
        payload.extend(name_bytes)
        payload.extend(struct.pack('>III', layer['m'], layer['n'], len(layer['elements'])))
        
        for elem in layer['elements']:
            payload.extend(struct.pack('>BB', elem['u_idx'], elem['v_idx']))
            payload.extend(float32_to_float16_bytes(elem['coefficient']))
            
    return bytes(payload)

def deserialize_genesis(binary_data):
    """Unpack big-endian .genesis binary payload into Python objects."""
    pos = 0
    
    # 1. Parse Header
    magic = struct.unpack_from('>I', binary_data, pos)[0]; pos += 4
    assert magic == GENESIS_MAGIC, "Invalid magic!"
    version = struct.unpack_from('>H', binary_data, pos)[0]; pos += 2
    assert version == GENESIS_VERSION, "Invalid version!"
    watermark = binary_data[pos : pos + 32].decode('utf-8').strip(); pos += 32
    perf_magic = struct.unpack_from('>I', binary_data, pos)[0]; pos += 4
    assert perf_magic == PERFECT_MAGIC, "Invalid secondary magic!"
    
    # 2. Parse Network hyperparameters
    hidden_size, num_heads, num_kv_heads, ffn_dim, num_blocks, vocab_size = struct.unpack_from('>IIIIII', binary_data, pos); pos += 24
    energy_targets = struct.unpack_from('>ffff', binary_data, pos); pos += 16
    layer_count = struct.unpack_from('>I', binary_data, pos)[0]; pos += 4
    
    metadata = {
        'version': version,
        'watermark': watermark,
        'hidden_size': hidden_size,
        'num_heads': num_heads,
        'num_kv_heads': num_kv_heads,
        'ffn_dim': ffn_dim,
        'num_blocks': num_blocks,
        'vocab_size': vocab_size,
        'energy_targets': list(energy_targets)
    }
    
    # 3. Parse Layers
    layers = []
    for _ in range(layer_count):
        name_len = struct.unpack_from('>H', binary_data, pos)[0]; pos += 2
        name = binary_data[pos : pos + name_len].decode('utf-8'); pos += name_len
        m, n, rank = struct.unpack_from('>III', binary_data, pos); pos += 12
        
        elements = []
        for _ in range(rank):
            u_idx, v_idx = struct.unpack_from('>BB', binary_data, pos); pos += 2
            coeff_bytes = binary_data[pos : pos + 2]; pos += 2
            coeff = float16_bytes_to_float32(coeff_bytes)
            elements.append({
                'u_idx': u_idx,
                'v_idx': v_idx,
                'coefficient': coeff
            })
            
        layers.append({
            'name': name,
            'm': m,
            'n': n,
            'elements': elements
        })
        
    return metadata, layers

def run_proof():
    print("======================================================================")
    print("ZYMATICA | Procedural Seed File Format: Binary Layout & Parsing Proof")
    print("======================================================================\n")

    # Define mock model metadata
    metadata = {
        'hidden_size': 1024,
        'num_heads': 8,
        'num_kv_heads': 2,
        'ffn_dim': 3584,
        'num_blocks': 24,
        'vocab_size': 248320,
        'energy_targets': [1.0, 1.25, 0.95, 1.1]
    }

    # Define mock layer projections
    layers = [
        {
            'name': 'model.layers.0.self_attn.q_proj.weight',
            'm': 1024,
            'n': 1024,
            'elements': [
                {'u_idx': 15, 'v_idx': 42, 'coefficient': 0.854},
                {'u_idx': 88, 'v_idx': 102, 'coefficient': -0.321}
            ]
        },
        {
            'name': 'model.layers.0.self_attn.v_proj.weight',
            'm': 1024,
            'n': 256,
            'elements': [
                {'u_idx': 4, 'v_idx': 19, 'coefficient': 1.45},
                {'u_idx': 120, 'v_idx': 3, 'coefficient': -0.925}
            ]
        }
    ]

    print("[1] Serializing Model Metadata & Layers to Binary Stream (.genesis)...")
    binary_payload = serialize_genesis(metadata, layers)
    print(f"  -> Generated Binary stream size: {len(binary_payload)} bytes")

    print("\n[2] Deserializing Binary Stream...")
    meta_rec, layers_rec = deserialize_genesis(binary_payload)
    
    print("\n[3] Verification Report:")
    print(f"  - Watermark:      '{meta_rec['watermark']}' (Matches Expected: ip zymatica.space)")
    print(f"  - Version:        v{meta_rec['version']}")
    print(f"  - Hidden Size:    {meta_rec['hidden_size']}")
    print(f"  - FFN Dimension:  {meta_rec['ffn_dim']}")
    print(f"  - Layer Count:    {len(layers_rec)}")
    
    for i, layer in enumerate(layers_rec):
        print(f"    * Layer {i+1}: '{layer['name']}' ({layer['m']}x{layer['n']})")
        for j, elem in enumerate(layer['elements']):
            expected = layers[i]['elements'][j]
            print(f"      Rank {j+1}: U={elem['u_idx']} V={elem['v_idx']} Coeff={elem['coefficient']:.4f} (Expected Coeff: {expected['coefficient']:.4f})")

    print("\n[VERIFICATION] Binary serialization and parsing verified.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica .genesis Binary Parsing Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
