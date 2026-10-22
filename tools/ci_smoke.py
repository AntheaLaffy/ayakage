#!/usr/bin/env python3
"""Exercise an extracted Linux release with a separately licensed test model."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import socket
import subprocess
import tarfile
import tempfile

from PIL import Image


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def run(command, log, env):
    with log.open("w", encoding="utf-8") as output:
        result = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT, env=env, timeout=120)
    if result.returncode:
        print(log.read_text(encoding="utf-8"))
        raise RuntimeError(f"command failed ({result.returncode}): {command}")


def pixels(path):
    with Image.open(path) as image:
        image = image.convert("RGBA")
        counts = image.getcolors(maxcolors=image.width * image.height)
        return {
            "size": list(image.size),
            "transparent": sum(count for count, p in counts if p[3] == 0),
            "visible": sum(count for count, p in counts if p[3] > 0),
            "green": sum(count for count, p in counts if p == (0, 255, 0, 255)),
            "colors": len(counts),
        }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, default=Path(".local/ci-result"))
    parser.add_argument("--max-glibc", help="maximum linked glibc symbol version, e.g. 2.35")
    parser.add_argument("--require-clean", action="store_true")
    args = parser.parse_args()
    archive_path = args.package.resolve()
    model = args.model.resolve(strict=True)
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    sums = [line.split() for line in archive_path.with_name("SHA256SUMS").read_text().splitlines()]
    require([line for line in sums if len(line) == 2 and line[1] == archive_path.name] ==
            [[digest(archive_path), archive_path.name]], "archive SHA-256 mismatch")

    with tempfile.TemporaryDirectory(prefix="unpacked-", dir=output) as temporary:
        unpacked = Path(temporary)
        with tarfile.open(archive_path) as archive:
            for entry in archive.getmembers():
                parts = PurePosixPath(entry.name)
                require(not parts.is_absolute() and ".." not in parts.parts, "unsafe archive path")
                require(entry.isdir() or entry.isfile(), "links and special files are not allowed")
                require(not {".git", ".local", "target", "dist", "__pycache__"}.intersection(parts.parts),
                        "local data or generated files in package")
            archive.extractall(unpacked)
        roots = list(unpacked.iterdir())
        require(len(roots) == 1 and roots[0].is_dir(), "expected one portable directory")
        bundle = roots[0]
        info = json.loads((bundle / "build-info.json").read_text())
        binary = bundle / "ayakage"
        require(digest(binary) == info["binary_sha256"], "binary SHA-256 mismatch")
        require(digest(bundle / "Cargo.lock") == info["cargo_lock_sha256"], "lockfile SHA-256 mismatch")
        require(os.access(binary, os.X_OK), "binary lost executable permission")
        require(json.loads((bundle / "licenses/index.json").read_text()), "missing license index")
        for name in ("Cargo.toml", "LICENSE", "src/main.rs", "assets/fonts/OFL.txt", ".github/workflows/ci.yml"):
            require((bundle / name).is_file(), f"missing source or notice: {name}")
        if args.require_clean:
            require(info["source_dirty"] is False, "release source is dirty")
            require(info["source_commit"] == os.environ.get("GITHUB_SHA"), "wrong source commit")

        version = subprocess.check_output([binary, "--version"], text=True).strip()
        require(version == f'ayakage {info["version"]}', "wrong executable version")
        symbols = subprocess.check_output(["readelf", "--version-info", binary], text=True)
        glibc = max(tuple(map(int, v.split("."))) for v in re.findall(r"GLIBC_([0-9.]+)", symbols))
        if args.max_glibc:
            require(glibc <= tuple(map(int, args.max_glibc.split("."))), f"glibc baseline too new: {glibc}")

        env = dict(os.environ, WAYLAND_DISPLAY="", XDG_CONFIG_HOME=str(output / "config"))
        run([binary, "--render-test", model, "--output-dir", output / "render"], output / "render.log", env)
        render = json.loads((output / "render/report.json").read_text())
        windows = {}
        for background in ("Transparent", "Green"):
            folder = output / background
            folder.mkdir(exist_ok=True)
            config = folder / "settings.json"
            config.write_text(json.dumps({
                "tracking": {"port": free_port()}, "background": background,
                "output_size": [960, 720], "output_decorated": False,
            }), encoding="utf-8")
            command = [
                "xvfb-run", "-a", "-s", "-screen 0 1600x1000x24", str(binary),
                "--demo", "--output", "--config", str(config), "--control-port", str(free_port()),
                "--screenshot", str(folder / "studio.png"), "--smoke-test",
            ]
            run([*command, "--model", model], folder / "native.log", env)
            require(pixels(folder / "studio.png")["colors"] > 50, "main window is blank")
            frame = pixels(folder / "studio-output.png")
            require(frame["colors"] > 50 and frame["visible"] > 1000, "output window is blank")
            if background == "Transparent":
                require(frame["transparent"] > 1000, "output lost transparency")
            else:
                require(frame["green"] > 1000, "green background is missing")
                require(frame["visible"] - frame["green"] > 1000, "model missing from green output")
            saved = json.loads(config.read_text())
            require(Path(saved["last_model"]).resolve() == model, "last model was not saved")
            require(saved["background"] == background, "output settings were not saved")
            # Start without --model to exercise real persisted startup state.
            run(command, folder / "restore.log", env)
            require(pixels(folder / "studio-output.png")["colors"] > 50, "restored model is blank")
            windows[background] = frame

        report = {
            "version": version, "source_commit": info["source_commit"], "source_dirty": info["source_dirty"],
            "archive_sha256": digest(archive_path), "binary_sha256": info["binary_sha256"],
            "maximum_glibc_symbol": ".".join(map(str, glibc)),
            "adapter": render["adapter"], "tracking_changed_pixels": render["tracking_changed_pixels"],
            "native_backend": "X11 / Xvfb", "windows": windows, "model_restored": True,
        }
        (output / "report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
