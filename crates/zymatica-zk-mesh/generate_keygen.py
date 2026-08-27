# Watermark: Pure Python Solana Keypair Generator (using pycryptodome)
import os
import json
from Crypto.PublicKey import ECC

def b58encode(v):
    alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    n = int.from_bytes(v, 'big')
    res = []
    while n > 0:
        n, r = divmod(n, 58)
        res.append(alphabet[r])
    for b in v:
        if b == 0:
            res.append(alphabet[0])
        else:
            break
    return "".join(reversed(res))

# 1. Generate Ed25519 key
key = ECC.generate(curve='ed25519')
priv_bytes = key._seed
pub_bytes = key.public_key().export_key(format='raw')

# 2. Form 64-byte keypair list (Solana standard format)
keypair_list = list(priv_bytes) + list(pub_bytes)

# 3. Create target directory
os.makedirs("target/deploy", exist_ok=True)

# 4. Save to target/deploy/zk_lorawan-keypair.json
output_path = "target/deploy/zk_lorawan-keypair.json"
with open(output_path, "w") as f:
    json.dump(keypair_list, f)

# 5. Output public key (Program ID)
pubkey_b58 = b58encode(pub_bytes)
print("=" * 60)
print(f"SUCCESS: Generated Solana Program Keypair!")
print(f"File Path:  {os.path.abspath(output_path)}")
print(f"Program ID: {pubkey_b58}")
print("=" * 60)
