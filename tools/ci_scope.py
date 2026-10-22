#!/usr/bin/env python3
"""Select full CI for code changes; keep required status available for docs."""
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess


DOCUMENTS = {
    "README.md", "CONTRIBUTING.md", "CHANGELOG.md", "AGENTS.md", "THIRD_PARTY.md", "talk.md",
    ".github/pull_request_template.md", "assets/fonts/README.md",
}


def is_document(path):
    return (path in DOCUMENTS or PurePosixPath(path).name == "CODEMAP.md"
            or (path.startswith("docs/") and path.endswith(".md")))


def classify(event_name, event, ref, force_full=False, cwd=None):
    if force_full:
        return True, "full CI explicitly requested", []
    if event_name == "pull_request":
        base = event["pull_request"]["base"]["sha"]
        head = event["pull_request"]["head"]["sha"]
    elif event_name == "push" and ref.startswith("refs/heads/"):
        base, head = event["before"], event["after"]
    else:
        return True, "release, schedule or manual run", []
    # Missing history must never turn into a documentation exemption.
    if not all(re.fullmatch(r"(?:[a-f0-9]{40}|[a-f0-9]{64})", sha) and set(sha) != {"0"}
               for sha in (base, head)):
        return True, "missing comparison commit", []
    try:
        if event_name == "pull_request":
            base = subprocess.check_output(["git", "merge-base", base, head], cwd=cwd, text=True).strip()
        changes = subprocess.check_output(
            ["git", "diff", "--name-only", "--no-renames", "-z", base, head, "--"], cwd=cwd
        )
    except subprocess.CalledProcessError:
        return True, "comparison history unavailable", []
    paths = [path.decode("utf-8", errors="surrogateescape") for path in changes.split(b"\0") if path]
    full = not paths or any(not is_document(path) for path in paths)
    return full, "code/configuration changed" if full else "documentation only", paths


def main():
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))
    full, reason, paths = classify(
        os.environ["GITHUB_EVENT_NAME"], event, os.environ["GITHUB_REF"],
        force_full=os.environ.get("FORCE_FULL") == "true",
    )
    print(json.dumps({"full": full, "reason": reason, "paths": paths}, ensure_ascii=True, indent=2))
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
        output.write(f"full={str(full).lower()}\n")


if __name__ == "__main__":
    main()
