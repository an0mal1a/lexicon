param(
    [string]$Dataset = "datasets/generated/es-dev.lxdb",
    [switch]$NoWeb
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Dataset)) {
    Write-Host "Compilando el dataset de desarrollo LXDB…"
    cargo run --manifest-path ..\lexinexo\Cargo.toml -p lxdb-cli -- compile .\datasets\fixtures\es-dev.lx -o $Dataset
}

$env:LEXICON_DATASET = $Dataset
Start-Process -WindowStyle Hidden -FilePath cargo -ArgumentList "run", "-p", "lexicon-api" -WorkingDirectory $PSScriptRoot\..
Write-Host "API: http://127.0.0.1:3001"

if (-not $NoWeb) {
    Start-Process -WindowStyle Hidden -FilePath npm.cmd -ArgumentList "run", "dev" -WorkingDirectory $PSScriptRoot\..\apps\web
    Write-Host "Web: http://localhost:3000"
}
