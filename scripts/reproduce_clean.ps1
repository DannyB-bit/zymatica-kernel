param(
    [string]$Image = "zymatica-verify:1.98.0"
)
$ErrorActionPreference = "Stop"
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    throw "Docker is required for a clean-container reproduction."
}
docker build -f Dockerfile.verify -t $Image .
docker run --rm $Image
Write-Host "PASS: clean Docker reproduction completed"
