import subprocess
import sys
import tempfile
import os
import shutil

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))

STYLE_CSS = os.path.join(REPO_ROOT, "web", "styles", "output.css")
COMPOSE_FILE = os.path.join(REPO_ROOT, "docker", "docker-compose-tailwind.yml")

with tempfile.TemporaryDirectory() as tmpdir:
    shutil.copy(os.path.join(REPO_ROOT, "web", "styles", "input.css"), tmpdir)

    pages_dir = os.path.join(REPO_ROOT, "web", "pages")

    result = subprocess.run(
        [
            "docker",
            "compose",
            "-f",
            COMPOSE_FILE,
            "run",
            "--rm",
            "-v",
            f"{tmpdir}:/app/styles",
            "-v",
            f"{pages_dir}:/app/pages",
            "tailwind",
        ],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print("Tailwind build failed:")
        print(result.stderr)
        sys.exit(1)

    out_file = os.path.join(tmpdir, "output.css")
    with open(out_file) as f:
        generated = f.read()

    with open(STYLE_CSS) as f:
        committed = f.read()

    if generated != committed:
        print("CSS is out of sync. Run ./cresco.sh update_css and commit the result.")
        sys.exit(1)

    print("CSS is up to date.")
