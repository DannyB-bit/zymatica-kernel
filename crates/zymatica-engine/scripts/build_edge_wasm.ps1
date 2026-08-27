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

Invoke-Checked "edge WASM release build" { cargo build --release --lib --target wasm32-unknown-unknown --no-default-features }
Invoke-Checked "edge WASM node instantiation proof" { node scripts\verify_edge_wasm.mjs target\wasm32-unknown-unknown\release\zymatica_core.wasm }

New-Item -ItemType Directory -Force deployment\edge-wasm | Out-Null
Copy-Item target\wasm32-unknown-unknown\release\zymatica_core.wasm deployment\edge-wasm\zymatica_engine.wasm -Force
Write-Host "edge_wasm=deployment\edge-wasm\zymatica_engine.wasm"
