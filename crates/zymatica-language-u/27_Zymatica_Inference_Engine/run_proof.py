# ZYMATICA: Zymatica Inference Engine (Class 27) Verification Run
# Watermark: ip zymatica.space | astronautshe.com
import os
import sys
import subprocess

def main():
    print("======================================================================")
    print("  ZYMATICA INFERENCE ENGINE (CLASS 27) VERIFICATION HARNESS")
    print("======================================================================\n")
    
    # Run the Python runtime proof inside the inventory as a sample verification
    script_dir = os.path.dirname(os.path.abspath(__file__))
    py_proof = os.path.join(script_dir, "zymatica-inference-engine-inventory", "zymatica-inference-engine-python", "proof.py")
    
    if not os.path.exists(py_proof):
        print(f"[-] Error: Python proof script not found at {py_proof}")
        sys.exit(1)
        
    print(f"[*] Launching Python Sub-Runtime Proof: {py_proof}")
    try:
        res = subprocess.run([sys.executable, py_proof], capture_output=True, text=True, timeout=15)
        print(res.stdout)
        if res.returncode == 0:
            print("[+] Class 27 (Zymatica Inference Engine) verified successfully!")
        else:
            print(f"[-] Verification failed:\n{res.stderr}")
            sys.exit(1)
    except Exception as e:
        print(f"[-] Execution exception: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
