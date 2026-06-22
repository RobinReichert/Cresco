#!/usr/bin/env pwsh

param(
    [Parameter(Position = 0)]
    [string]$Command,

    [Parameter(Position = 1)]
    [string]$Target
)

$ErrorActionPreference = "Stop"

$PARTITIONS_PATH = "build/app_embedded/partitions.csv"
$APP_EMBEDDED_ELF_PATH = "build/app_embedded/riscv32imc-unknown-none-elf/release/cross"
$CAMERA_ELF_PATH = "build/camera/camera.elf"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$LocalConfig = Join-Path $ScriptDir "cresco.local.ps1"
if (Test-Path $LocalConfig) {
    . $LocalConfig
}

function Run-InIdf {
    param([string]$InnerCommand)

    if ([string]::IsNullOrEmpty($IDF_EXPORT_CMD)) {
        Write-Error "ESP-IDF activation not configured.`nCopy cresco.local.ps1.example to cresco.local.ps1 and set IDF_EXPORT_CMD."
        exit 1
    }
    & pwsh -NoProfile -Command "$IDF_EXPORT_CMD | Out-Null; $InnerCommand"
}

function Show-Help {
    @"
Usage: cresco.ps1 <command>

Commands:

  serve_docs
      Start the documentation server using Docker Compose.

  build_css
      Build the tailwind css file with all used css classes.

  setup
      Sync dependencies, install the git pre-commit hook, and pull the
      drawio-export image.

  format
      Format the Rust code using cargo fmt.

  test
      Run tests for the 'logic' package.

  build {app_embedded|camera}
      app_embedded  Build the embedded Rust application in release mode.
      camera        Build the ESP-IDF camera firmware via idf.py.

  flash {app_embedded|camera}
      Flash the compiled firmware to the device using espflash.

  monitor {app_embedded|camera}
      Open a serial monitor for the device.

  -h, --help
      Show this help message.

Examples:

  cresco.ps1 setup
  cresco.ps1 build camera
  cresco.ps1 flash app_embedded
"@
}

switch ($Command) {
    { $_ -in @("-h", "--help", "help") } {
        Show-Help
    }
    "serve_docs" {
        docker compose -f docker/docker-compose-docs-server.yml run --rm drawio-export
        docker compose -f docker/docker-compose-docs-server.yml run --rm --service-ports docs-server
    }
    "build_css" {
        docker compose -f docker/docker-compose-tailwind.yml run --rm tailwind
    }
    "setup" {
        uv sync
        uv run pre-commit install
        docker pull rlespinasse/drawio-export
    }
    "format" {
        cargo fmt
    }
    "test" {
        cargo test -p logic
    }
    "build" {
        switch ($Target) {
            "app_embedded" {
                Push-Location cross
                try {
                    cargo build --release
                }
                finally {
                    Pop-Location
                }
                Copy-Item partitions.csv $PARTITIONS_PATH
            }
            "camera" {
                Run-InIdf "idf.py -C camera -B build/camera build"
            }
            default {
                Write-Host "Usage: cresco.ps1 build {app_embedded|camera}"
                exit 1
            }
        }
    }
    "flash" {
        switch ($Target) {
            "app_embedded" {
                espflash flash --partition-table $PARTITIONS_PATH $APP_EMBEDDED_ELF_PATH
            }
            "camera" {
                espflash flash $CAMERA_ELF_PATH
            }
            default {
                Write-Host "Usage: cresco.ps1 flash {app_embedded|camera}"
                exit 1
            }
        }
    }
    "monitor" {
        switch ($Target) {
            "app_embedded" {
                espflash monitor --chip esp32c3 --log-format defmt --elf $APP_EMBEDDED_ELF_PATH
            }
            "camera" {
                espflash monitor --chip esp32s3 --elf $CAMERA_ELF_PATH
            }
            default {
                Write-Host "Usage: cresco.ps1 monitor {app_embedded|camera}"
                exit 1
            }
        }
    }
    default {
        Show-Help
        exit 1
    }
}
