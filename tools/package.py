#!/usr/bin/env python3
"""Build a Linux bundle with rebuildable source, notices and optional archive."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tempfile


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_output(root, *args):
    result = subprocess.run(
        ["git", *args], cwd=root, capture_output=True, text=True, check=False
    )
    return result.stdout.strip() if result.returncode == 0 else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", action="store_true", help="also create tar.gz and SHA256SUMS")
    args = parser.parse_args()
    if platform.system() != "Linux":
        parser.error("this bundle supports Linux hosts only")

    root = Path(__file__).resolve().parents[1]
    subprocess.run(["cargo", "build", "--locked", "--release"], cwd=root, check=True)
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1"], cwd=root
    ))
    project = next(p for p in metadata["packages"] if Path(p["manifest_path"]) == root / "Cargo.toml")
    dist = root / "dist"
    dist.mkdir(exist_ok=True)
    output = dist / f"kage-studio-linux-{platform.machine()}"
    if output.is_symlink():
        raise RuntimeError(f"refusing to replace symlink: {output}")

    # Stage a fresh tree so retired files cannot survive a later release.
    with tempfile.TemporaryDirectory(prefix="package-", dir=dist) as temporary:
        staging = Path(temporary) / output.name
        staging.mkdir()
        shutil.copy2(Path(metadata["target_directory"]) / "release/kage-studio", staging / "kage-studio")
        for name in (
            "README.md", "LICENSE", "THIRD_PARTY.md", "CONTRIBUTING.md", "CHANGELOG.md",
            "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "AGENTS.md", "CODEMAP.md",
            "talk.md", ".gitignore",
        ):
            shutil.copy2(root / name, staging / name)
        for name in ("docs", "src", "assets", "tools", ".github"):
            shutil.copytree(root / name, staging / name, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))

        licenses = staging / "licenses"
        licenses.mkdir()
        index = []
        for package in metadata["packages"]:
            folder = Path(package["manifest_path"]).parent
            candidates = [
                path for path in folder.iterdir()
                if path.is_file() and path.name.upper().startswith(
                    ("LICENSE", "LICENCE", "NOTICE", "COPYING", "COPYRIGHT")
                )
            ]
            if package.get("license_file"):
                candidates.append(folder / package["license_file"])
            for source in set(candidates):
                destination = licenses / f'{package["name"]}-{package["version"]}' / source.name
                destination.parent.mkdir(exist_ok=True)
                shutil.copy2(source, destination)
            index.append({key: package.get(key) for key in ("name", "version", "license", "repository")})
        shutil.copy2(root / "assets/fonts/OFL.txt", licenses / "NotoCJK-OFL.txt")
        (licenses / "index.json").write_text(json.dumps(index, ensure_ascii=False, indent=2), encoding="utf-8")
        status = git_output(root, "status", "--porcelain", "--untracked-files=normal")
        build_info = {
            "version": project["version"],
            "source_commit": git_output(root, "rev-parse", "HEAD"),
            "source_dirty": bool(status) if status is not None else None,
            "binary_sha256": sha256(staging / "kage-studio"),
            "cargo_lock_sha256": sha256(staging / "Cargo.lock"),
            "host_arch": platform.machine(),
            "libc": platform.libc_ver(),
            "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
            "source_included": True,
        }
        (staging / "build-info.json").write_text(json.dumps(build_info, indent=2), encoding="utf-8")
        if output.exists():
            shutil.rmtree(output)
        shutil.move(staging, output)

    if args.archive:
        archive = dist / f'kage-studio-{project["version"]}-linux-{platform.machine()}.tar.gz'
        with tarfile.open(archive, "w:gz") as bundle:
            bundle.add(output, arcname=output.name)
        (dist / "SHA256SUMS").write_text(f"{sha256(archive)}  {archive.name}\n", encoding="utf-8")
        print(archive)
    print(output)


if __name__ == "__main__":
    main()
