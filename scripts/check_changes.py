import subprocess
import sys
import os
import re

base_branch = sys.argv[1]
patterns = sys.argv[2:]

print(f"Base branch: {base_branch}")

subprocess.run(["git", "fetch", "origin", base_branch], check=True)

result = subprocess.run(
    ["git", "diff", "--name-only", f"origin/{base_branch}...HEAD"],
    capture_output=True,
    text=True,
    check=True,
)

changed_files = result.stdout.strip()
print("Changed files:")
print(changed_files)

pattern = "^(" + "|".join(patterns) + ")"
print(f"Looking for changes matching pattern: {pattern}")

result = re.search(pattern, changed_files, re.MULTILINE)

print(f"Found: {result}")

changed = bool(result)

with open(os.environ["GITHUB_OUTPUT"], "a") as f:
    f.write(f"changed={str(changed).lower()}\n")
