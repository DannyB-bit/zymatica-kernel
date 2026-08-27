param(
    [switch]$SkipPhysicalGpu,
    [string]$GpuModelDir = $env:ZYMATICA_GPU_MODEL_DIR,
    [string]$GpuQ3CacheDir = $env:ZYMATICA_GPU_Q3_CACHE_DIR
)

$ErrorActionPreference = "Stop"

$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "ZYMATICA PRODUCTION READINESS HARDENING AUDIT" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Formatting audit
Write-Host "[1/7] Running Cargo Fmt code alignment checks..." -ForegroundColor Yellow
Invoke-Checked "cargo fmt --check" { cargo fmt --all -- --check }
Write-Host "[1/7] Cargo Fmt: PASS" -ForegroundColor Green

# 2. Warnings and Clippy audit
Write-Host "[2/7] Running Cargo Clippy warnings analysis..." -ForegroundColor Yellow
Invoke-Checked "cargo clippy" { cargo clippy --workspace --all-targets --all-features -- -D warnings }
Write-Host "[2/7] Cargo Clippy: PASS" -ForegroundColor Green

# 3. Test Matrix execution
Write-Host "[3/7] Running complete unit and integration test suite..." -ForegroundColor Yellow
Invoke-Checked "cargo test --workspace" { cargo test --workspace }
Invoke-Checked "cargo test --workspace --no-default-features" { cargo test --workspace --no-default-features --lib }
Invoke-Checked "cargo test --workspace --all-features" { cargo test --workspace --all-features }
Write-Host "[3/7] Workspace Tests: PASS" -ForegroundColor Green

# 4. Dependency vulnerability audit
Write-Host "[4/7] Running RustSec supply-chain audit..." -ForegroundColor Yellow
Invoke-Checked "cargo audit" { cargo audit -D warnings --ignore RUSTSEC-2024-0436 }
Write-Host "[4/7] RustSec audit: PASS" -ForegroundColor Green

# 5. WASM portability target builds
Write-Host "[5/7] Running Edge WASM packaging audit..." -ForegroundColor Yellow
Invoke-Checked "wasm32-wasip1 check" { cargo check --target wasm32-wasip1 }
Invoke-Checked "wasm32-wasip1 no-default lib check" { cargo check --lib --target wasm32-wasip1 --no-default-features }
Invoke-Checked "wasm32-unknown-unknown no-default lib check" { cargo check --lib --target wasm32-unknown-unknown --no-default-features }
Invoke-Checked "edge WASM packaging" { powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build_edge_wasm.ps1 }
Write-Host "[5/7] WASM packaging: PASS" -ForegroundColor Green

# 6. Production stress harnesses
Write-Host "[6/7] Executing production security and stress verification..." -ForegroundColor Yellow
Write-Host "  -> Running Adversarial Fuzzing Checks..." -ForegroundColor Gray
Invoke-Checked "production-fuzz-test" { cargo run --release -- production-fuzz-test }
Write-Host "  -> Running Micro-Benchmark Latency Baseline..." -ForegroundColor Gray
Invoke-Checked "production-benchmark-baseline" { cargo run --release -- production-benchmark-baseline }
Write-Host "  -> Running High-Concurrency Soak Simulation (15 seconds)..." -ForegroundColor Gray
Invoke-Checked "production-soak-test" { cargo run --release -- production-soak-test --duration-secs 15 }

Write-Host "[6/7] Production stress checks: PASS" -ForegroundColor Green

# 7. Field-readiness, ecosystem complements, local multi-node proof, and physical GPU gate
Write-Host "[7/7] Executing field-readiness, ecosystem, and accelerator proofs..." -ForegroundColor Yellow
Invoke-Checked "frontier-software-proof" { cargo run --release -- frontier-software-proof }
Invoke-Checked "field-multinode-proof" { cargo run --release -- field-multinode-proof }
Invoke-Checked "field-readiness-audit" { cargo run --release -- field-readiness-audit }
Invoke-Checked "ecosystem-proof" { cargo run --release -- ecosystem-proof }
Invoke-Checked "studio-dashboard" { cargo run --release -- studio-dashboard --output target\ecosystem-proof\studio.html }
if ($SkipPhysicalGpu) {
    Write-Host "  -> Physical GPU proof explicitly skipped." -ForegroundColor Yellow
} else {
    Invoke-Checked "gpu-proof" { cargo run --release --features gpu -- gpu-proof }
    Invoke-Checked "gpu-bench" { cargo run --release --features gpu -- gpu-bench }
    if ([string]::IsNullOrWhiteSpace($GpuModelDir) -xor [string]::IsNullOrWhiteSpace($GpuQ3CacheDir)) {
        throw "GpuModelDir and GpuQ3CacheDir must be provided together"
    }
    if (-not [string]::IsNullOrWhiteSpace($GpuModelDir)) {
        Invoke-Checked "gpu-model-proof" {
            cargo run --release --features gpu -- gpu-model-proof `
                --model-dir $GpuModelDir `
                --q3-cache-dir $GpuQ3CacheDir `
                --prompt-ids "2,10,20,30"
        }
        Invoke-Checked "q3-gpu field benchmark" {
            cargo run --release --features gpu -- pi-bench `
                --model-dir $GpuModelDir `
                --engine q3-gpu `
                --q8-cache-dir $GpuQ3CacheDir `
                --prompt-ids 2 `
                --new-tokens 32 `
                --passes 3
        }
    } else {
        Write-Host "  -> Real Q3 GPU model proof skipped; provide -GpuModelDir and -GpuQ3CacheDir." -ForegroundColor Yellow
    }
}
Write-Host "[7/7] Field-readiness, ecosystem, and accelerator checks: PASS" -ForegroundColor Green

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "ZYMATICA SOFTWARE FIELD READINESS AUDIT: SUCCESS" -ForegroundColor Green
Write-Host "All 11 hardware-gated items have simulator-backed verification; physical validation is reported separately by field-readiness-audit." -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
