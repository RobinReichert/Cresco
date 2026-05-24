import subprocess
import sys
import tempfile
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))

STYLE_CSS = os.path.join(REPO_ROOT, "static", "style.css")
COMPOSE_FILE = os.path.join(REPO_ROOT, "docker", "docker-compose-tailwind-minify.yml")

with tempfile.TemporaryDirectory() as tmpdir:
    out_file = os.path.join(tmpdir, "output.css")

    result = subprocess.run(
        [
            "docker",
            "compose",
            "-f",
            COMPOSE_FILE,
            "run",
            "--rm",
            "-v",
            f"{tmpdir}:/app/out",
            "tailwind",
        ],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print("Tailwind build failed:")
        print(result.stderr)
        sys.exit(1)

    with open(out_file) as f:
        generated = f.read()

    with open(STYLE_CSS) as f:
        committed = f.read()

    if generated != committed:
        print("CSS is out of sync. Run ./cresco.sh update_css and commit the result.")
        sys.exit(1)

    print("CSS is up to date.")
