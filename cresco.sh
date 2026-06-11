#!/bin/bash

PARTITIONS_PATH="build/app_embedded/partitions.csv"
APP_EMBEDDED_ELF_PATH="build/app_embedded/riscv32imc-unknown-none-elf/release/cross"
CAMERA_ELF_PATH="build/camera/camera.elf"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[ -f "$SCRIPT_DIR/cresco.local.sh" ] && . "$SCRIPT_DIR/cresco.local.sh"

run_in_idf() {
  if [ -z "${IDF_EXPORT_CMD:-}" ]; then
    echo "ESP-IDF activation not configured." >&2
    echo "Copy cresco.local.sh.example to cresco.local.sh and set IDF_EXPORT_CMD." >&2
    exit 1
  fi
  bash -c "${IDF_EXPORT_CMD} >/dev/null && $1" bash
}

show_help() {
  cat << EOF
Usage: $0 <command>

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

  $0 setup
  $0 build camera
  $0 flash app_embedded
EOF
}

case "$1" in
    -h|--help)
        show_help
        ;;
    serve_docs)
        docker compose -f docker/docker-compose-docs-server.yml run --rm drawio-export
        docker compose -f docker/docker-compose-docs-server.yml run --rm --service-ports docs-server
        ;;
    build_css)
        docker compose -f docker/docker-compose-tailwind.yml run --rm tailwind
        ;;
    setup)
        uv sync
        uv run pre-commit install
        docker pull rlespinasse/drawio-export
        ;;
    format)
        cargo fmt
        ;;
    test)
        cargo test -p logic
        ;;
    build)
        case "$2" in
            app_embedded)
                cd cross
                cargo build --release && cd .. && cp partitions.csv $PARTITIONS_PATH
                ;;
            camera)
                run_in_idf "idf.py -C camera -B build/camera build"
                ;;
            *)
                echo "Usage: $0 build {app_embedded|camera}"
                exit 1
                ;;
        esac
        ;;
    flash)
        case "$2" in
            app_embedded)
                espflash flash --partition-table $PARTITIONS_PATH $ELF_PATH
                ;;
            camera)
                espflash flash $CAMERA_ELF_PATH
                ;;
            *)
                echo "Usage: $0 flash {app_embedded|camera}"
                exit 1
                ;;
        esac
        ;;
    monitor)
        case "$2" in
            app_embedded)
                espflash monitor --chip esp32c3 --log-format defmt --elf $APP_EMBEDDED_ELF_PATH
                ;;
            camera)
                espflash monitor --chip esp32s3 --elf $CAMERA_ELF_PATH
                ;;
            *)
                echo "Usage: $0 monitor {app_embedded|camera}"
                exit 1
                ;;
        esac
        ;;
    *)
        show_help
        exit 1
        ;;
esac
