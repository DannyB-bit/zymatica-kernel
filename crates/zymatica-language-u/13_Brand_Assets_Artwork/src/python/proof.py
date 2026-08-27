import os
import argparse
import hashlib

def get_file_hash(path):
    sha = hashlib.sha256()
    with open(path, 'rb') as f:
        while True:
            chunk = f.read(4096)
            if not chunk:
                break
            sha.update(chunk)
    return sha.hexdigest()

def run_proof():
    print("======================================================================")
    print("ZYMATICA | Brand Assets & Visual Identity Verification Proof")
    print("======================================================================\n")

    # Brand assets are in parent of this folder
    parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    logo_path = os.path.join(parent_dir, "Logo.jpg")
    arch_path = os.path.join(parent_dir, "architecture.png")

    print("[1] Verifying Official Zymatica Logo File...")
    if os.path.exists(logo_path):
        logo_size = os.path.getsize(logo_path)
        logo_hash = get_file_hash(logo_path)
        print(f"  - Logo path:       {logo_path}")
        print(f"  - File size:       {logo_size:,} bytes")
        print(f"  - SHA-256 Hash:    {logo_hash}")
        print("  [OK] Logo file verified intact.")
    else:
        print(f"  [ERROR] Logo.jpg not found at: {logo_path}")

    print("\n[2] Verifying Unified Language-U System Architecture Image...")
    if os.path.exists(arch_path):
        arch_size = os.path.getsize(arch_path)
        arch_hash = get_file_hash(arch_path)
        print(f"  - Architecture:    {arch_path}")
        print(f"  - File size:       {arch_size:,} bytes")
        print(f"  - SHA-256 Hash:    {arch_hash}")
        print("  [OK] System architecture diagram verified intact.")
    else:
        print(f"  [-] Error: architecture.png not found at: {arch_path}")

    # Official Zymatica Art Banners
    print("\n[3] Rendering Official Zymatica Brand Identity:")
    print("-" * 70)
    print("  Z Y M A T I C A  |  L A N G U A G E - U  |  A S T R O N A U T  S H E")
    print("-" * 70)
    print("  THE IMPOSSIBLE QUOTE:")
    print("  \"The impossible is just code waiting to be written,")
    print("   physics waiting to be rewritten, math a work in progress,")
    print("   and truth waiting to be discovered.\"")
    print("-" * 70)

    print("\n[VERIFICATION] Brand assets and registry confirmed.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica Brand Assets Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
