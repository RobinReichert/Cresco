import argparse
import sys
import subprocess
import shutil
from pathlib import Path
import os

PARTITIONS_PATH = "build/app_embedded/partitions.csv"
APP_EMBEDDED_ELF_PATH = "build/app_embedded/riscv32imc-unknown-none-elf/release/cross"
CAMERA_ELF_PATH = "build/camera/camera.elf"
ROOT = Path(__file__).resolve().parent.parent


def run(command, cwd: Path = ROOT, shell: bool = False):
    result = subprocess.run(command, cwd=cwd, shell=shell)
    if result.returncode != 0:
        sys.exit(result.returncode)


def run_in_idf(command: str):
    export_cmd = os.environ.get("IDF_EXPORT_CMD")
    if not export_cmd:
        sys.exit(
            "ESP-IDF activation not configured. Set IDF_EXPORT_CMD in cresco.local.sh / cresco.local.ps1."
        )
    if os.name == "nt":
        run(f"{export_cmd} && {command}", shell=True)  # cmd.exe, export.bat
    else:
        run(["bash", "-c", f"{export_cmd} && {command}"])


def main():
    parser = argparse.ArgumentParser(prog="cresco")
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("format", help="Format the Rust code using cargo fmt.")
    commands.add_parser("test", help="Run tests for the 'logic' package.")
    commands.add_parser(
        "serve_docs", help="Start the documentation server using Docker Compose."
    )
    commands.add_parser(
        "build_css", help="Build the tailwind css file with all used css classes."
    )
    commands.add_parser(
        "setup",
        help="Install the git pre-commit hook, and pull the drawio-export image.",
    )
    for keyword in ["build", "flash", "monitor"]:
        command = commands.add_parser(keyword, help=f"{keyword} app")
        command.add_argument("target", choices=["app_embedded", "camera"])
    args = parser.parse_args()

    match args.command:
        case "format":
            run(["cargo", "fmt"])

        case "test":
            run(["cargo", "test", "-p", "logic"])

        case "serve_docs":
            run(
                [
                    "docker",
                    "compose",
                    "-f",
                    "docker/docker-compose-docs-server.yml",
                    "run",
                    "--rm",
                    "drawio-export",
                ]
            )
            run(
                [
                    "docker",
                    "compose",
                    "-f",
                    "docker/docker-compose-docs-server.yml",
                    "run",
                    "--rm",
                    "--service-ports",
                    "docs-server",
                ]
            )

        case "build_css":
            run(
                [
                    "docker",
                    "compose",
                    "-f",
                    "docker/docker-compose-tailwind.yml",
                    "run",
                    "--rm",
                    "tailwind",
                ]
            )

        case "setup":
            run(["uv", "run", "pre-commit", "install"])
            run(["docker", "pull", "rlespinasse/drawio-export"])

        case "build":
            if args.target == "app_embedded":
                run(["cargo", "build", "--release"], cwd=ROOT.joinpath("cross"))
                shutil.copy(
                    ROOT.joinpath("partitions.csv"), ROOT.joinpath(PARTITIONS_PATH)
                )

            else:
                run_in_idf("idf.py -C camera -B build/camera build")

        case "flash":
            if args.target == "app_embedded":
                run(
                    [
                        "probe-rs",
                        "download",
                        "--chip",
                        "esp32c3",
                        "--idf-partition-table",
                        ROOT.joinpath(PARTITIONS_PATH),
                        ROOT.joinpath(APP_EMBEDDED_ELF_PATH),
                    ]
                )
                run(
                    [
                        "probe-rs",
                        "reset",
                        "--chip",
                        "esp32c3",
                    ]
                )

            else:
                run(["espflash", "flash", ROOT.joinpath(CAMERA_ELF_PATH)])

        case "monitor":
            if args.target == "app_embedded":
                run(
                    [
                        "probe-rs",
                        "attach",
                        "--chip",
                        "esp32c3",
                        ROOT.joinpath(APP_EMBEDDED_ELF_PATH),
                    ]
                )
            else:
                run(
                    [
                        "espflash",
                        "monitor",
                        "--chip",
                        "esp32s3",
                        "--elf",
                        ROOT.joinpath(CAMERA_ELF_PATH),
                    ]
                )

        case _:
            raise AssertionError(f"unhandled command: {args.command}")


if __name__ == "__main__":
    main()
