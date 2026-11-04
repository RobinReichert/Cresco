import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    return subprocess.run(
        [ROOT / "cresco.sh", "flash", "app_embedded"], cwd=ROOT
    ).returncode


if __name__ == "__main__":
    sys.exit(main())
