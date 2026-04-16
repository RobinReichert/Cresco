#!/bin/bash

ELF_PATH="build/app_embedded/riscv32imc-unknown-none-elf/release/cross"

case "$1" in
  -h|--help)
      show_help()
    ;;
  serve_docs)
    docker compose -f docker/docker-compose-docs-server.yml run --rm --service-ports docs-server
    ;;
  setup)
    if cmp -s scripts/pre-commit .git/hooks/pre-commit; then
      echo "Pre-commit hook already up to date"
    else
      cp scripts/pre-commit .git/hooks/pre-commit
      chmod +x .git/hooks/pre-commit
      echo "Pre-commit hook installed/updated!"
    fi
    ;;
  format)
      cargo fmt
    ;;
  test)
      cargo test -p logic
    ;;
  build_app_embedded)
      cd cross
      cargo build --release
    ;;
  flash)
      espflash flash $ELF_PATH
    ;;
  monitor)
      espflash monitor --chip esp32c3 --log-format defmt --elf $ELF_PATH
    ;;
  *)
    echo "Usage: $0 {serve_docs}"
    exit 1
    ;;
esac

show_help() {
  cat << EOF
Usage: $0 <command>

Commands:

  serve_docs
      Start the documentation server using Docker Compose.

  setup
      Install or update the git pre-commit hook.

  format
      Format the Rust code using cargo fmt.

  test
      Run tests for the 'logic' package.

  build_app_embedded
      Build the embedded application in release mode.

  flash
      Flash the compiled ELF to the ESP32-C3 device.

  monitor
      Open a serial monitor with defmt logs for the ESP32-C3.

  -h, --help
      Show this help message.

Examples:

  $0 setup
  $0 build_app_embedded
  $0 flash
EOF
}
