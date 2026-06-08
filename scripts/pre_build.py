import subprocess
import sys
import yaml
import json
import os
import re

GREEN = "\033[42m"
RED = "\033[41m"
SKIP = "\033[48;5;240m"
RESET = "\033[0m"

CACHE_FILE = ".pre-build-cache.json"
BASE_DIR = os.path.pardir
DEFAULT_FILES = ["."]
DEFAULT_EXCLUDES = [r"\.venv"]


def load_cache() -> dict[str, float]:
    try:
        with open(CACHE_FILE) as f:
            return json.load(f)
    except FileNotFoundError:
        return {}


def save_cache(cache: dict[str, float]):
    with open(CACHE_FILE, "w") as f:
        json.dump(cache, f)


def files_changed(files: list[str], cache: dict[str, float]) -> bool:
    for file in files + DEFAULT_FILES:
        mtime = os.path.getmtime(file)
        if cache.get(file) != mtime:
            return True
    return False


def update_cache(files: list[str], cache: dict[str, float]):
    for file in files:
        cache[file] = os.path.getmtime(file)


def find_files(patterns: list[str], excludes: list[str]) -> list[str]:
    matches = set()
    exclude_regexes = [re.compile(e) for e in (DEFAULT_EXCLUDES + excludes)]
    for pattern in patterns:
        regex = re.compile(pattern)
        for root, _, files in os.walk(BASE_DIR):
            for file in files:
                path = os.path.join(root, file)
                rel_path = os.path.relpath(path, BASE_DIR)
                if regex.search(rel_path):
                    if not any(e.search(rel_path) for e in exclude_regexes):
                        matches.add(path)
    return list(matches)


with open("../.pre-build-config.yaml") as f:
    config = yaml.safe_load(f)

cache = load_cache()
new_cache = cache.copy()
failed = False

for check in config["checks"]:
    name = check["name"]
    entry = check["entry"]
    patterns = check.get("files", [])
    print(f"{name}".ljust(50, "."), end="")
    files = []
    if len(patterns) > 0:
        excludes = check.get("exclude", [])
        files = find_files(patterns, excludes)

        if not files_changed(files, cache):
            print(f"{SKIP} SKIPPED {RESET}")
            continue

    result = subprocess.run(entry, shell=True, capture_output=True)
    if result.returncode != 0:
        print(f"{RED} FAILED  {RESET}")
        failed = True
    else:
        print(f"{GREEN} PASSED  {RESET}")
        update_cache(files, new_cache)

if failed:
    print("\nOne or more checks failed. Aborting build.")
    sys.exit(1)

update_cache(DEFAULT_FILES, new_cache)
save_cache(new_cache)
print("\nAll checks passed.")
sys.exit(0)
