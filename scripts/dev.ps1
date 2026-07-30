param(
    [string]$Dataset = "datasets/generated/es/dictionary.lxdb",
    [switch]$NoWeb
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Dataset)) {
    Write-Host "Generando el dataset español amplio con LXDB…"
    cargo run --manifest-path ..\lxdb\Cargo.toml -p lxdb-cli -- dictionary build es --profile game --config ..\lxdb\config\dictionaries\es.toml --output (Split-Path $Dataset -Parent)
}

$env:LEXICON_DATASET = $Dataset
Start-Process -WindowStyle Hidden -FilePath cargo -ArgumentList "run", "-p", "lexicon-api" -WorkingDirectory $PSScriptRoot\..
Write-Host "API: http://127.0.0.1:3001"

if (-not $NoWeb) {
    Start-Process -WindowStyle Hidden -FilePath npm.cmd -ArgumentList "run", "dev" -WorkingDirectory $PSScriptRoot\..\apps\web
    Write-Host "Web: http://localhost:3000"
}
