import os
import subprocess
import sys


def main() -> int:
    process = subprocess.Popen(
        ["mpremote", "connect", os.environ["MP_PORT"], "run", sys.argv[1]],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    passed = False
    for line in process.stdout:
        print(line)
        passed = passed or line.strip() == "PASS"
    process.wait()
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
