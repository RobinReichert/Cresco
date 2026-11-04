#!/usr/bin/env pwsh

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$LocalConfig = Join-Path $ScriptDir "cresco.local.ps1"
if (Test-Path $LocalConfig) {
    . $LocalConfig
}
$env:IDF_EXPORT_CMD = $IDF_EXPORT_CMD

uv run --project $ScriptDir (Join-Path $ScriptDir "scripts/cresco.py") @args
exit $LASTEXITCODE
