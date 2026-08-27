# Windows PowerShell Orchestration & Verification Script
# Watermark: ip zymatica.space | astronautshe.com
# WASM U-Performance Record verify loop

$ErrorActionPreference = "Stop"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  [+] INITIALIZING SKEPTIC-PROOF COMPILATION & VERIFICATION PIPELINE" -ForegroundColor Cyan
Write-Host "  [+] TARGET WORKSPACE: WASM_U-Performance_Record" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

# 1. Compile proof.zig to proof_wasm.wasm
Write-Host ""
Write-Host "[1/5] Compiling proof.zig to freestanding WebAssembly..." -ForegroundColor Green
if (!(Get-Command zig -ErrorAction SilentlyContinue)) {
    Write-Error "Zig compiler is missing from PATH. Please install Zig (https://ziglang.org) to run this script."
}

# Compile wasm
zig build-exe proof.zig -target wasm32-freestanding -O ReleaseFast --name proof_wasm --export=wasm_encode --export=wasm_get_encoded_bits --export=wasm_decode --export=run_verification
if (Test-Path "proof_wasm.wasm") {
    $wasmSize = (Get-Item "proof_wasm.wasm").Length
    Write-Host "  [+] WebAssembly binary built successfully! Size: $wasmSize bytes (~$([Math]::Round($wasmSize/1024, 2)) KB)" -ForegroundColor Green
} else {
    Write-Error "WASM compilation failed!"
}

# 2. Compile proof.zig to assembly for inspection
Write-Host ""
Write-Host "[2/5] Compiling proof.zig to native assembly (.s) for register audit..." -ForegroundColor Green
zig build-exe proof.zig -O ReleaseFast -femit-asm --cache-dir ./zig-cache
if (Test-Path "proof.s") {
    Write-Host "  [+] Native assembly dump generated successfully at proof.s" -ForegroundColor Green
} else {
    Write-Host "  [*] Assembly generation skipped or not supported on this platform target." -ForegroundColor Yellow
}

# Run WebAssembly binary structure audit
Write-Host ""
Write-Host "  [+] Executing WASM structure inspector..." -ForegroundColor Green
python proof_wasm_inspector.py
if (Test-Path "proof_wasm_structure.txt") {
    Write-Host "  [+] WASM binary structure audit report generated successfully at proof_wasm_structure.txt" -ForegroundColor Green
}

# 3. Run Node.js execution check
Write-Host ""
Write-Host "[3/5] Instantiating WASM inside Node.js CLI Benchmark..." -ForegroundColor Green
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Error "Node.js is missing from PATH. Node is required to run the WASM benchmark."
}
node proof.js

# 4. Run Python Parity Verification Loop
Write-Host ""
Write-Host "[4/5] Starting Cross-Runtime Bit-Parity Fuzzer (Python vs WASM)..." -ForegroundColor Green
if (!(Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Error "Python 3 is missing from PATH. Python is required to run parity tests."
}
python proof.py --fuzz

# 5. Launch local server for interactive sandbox
Write-Host ""
Write-Host "[5/5] Launching local HTTP Server for Interactive Browser Dashboard..." -ForegroundColor Green
Write-Host "  [!] Server hosting at http://localhost:8080/" -ForegroundColor Yellow
Write-Host "  [!] Pasting intent coordinates will compile and decompress them in real-time." -ForegroundColor Yellow
Write-Host "  [!] PRESS CTRL+C TO TERMINATE SERVER PROCESS." -ForegroundColor Red
Write-Host ""

python server.py
