#!/bin/bash

ELF_PATH="build/app_embedded/riscv32imc-unknown-none-elf/release/cross"

case "$1" in
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
