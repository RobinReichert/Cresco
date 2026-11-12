import os
import subprocess
import sys


def main() -> int:
    process = subprocess.Popen(
        [
            "mpremote",
            "connect",
            os.environ["MP_PORT"],
            "soft-reset",
            "run",
            sys.argv[1],
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    passed = False
    output_file = os.environ.get("GITHUB_OUTPUT")
    for line in process.stdout:
        print(line)
        passed = passed or line.strip() == "PASS"
        if output_file and line.startswith("OUTPUT "):
            with open(output_file, "a") as f:
                f.write(line.removeprefix("OUTPUT ").strip() + "\n")
    process.wait()
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
