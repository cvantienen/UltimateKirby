#!/usr/bin/env python3
"""Build and install the Workbench macOS app without leaving local bundle copies."""
from __future__ import annotations

import argparse
import importlib
import os
import platform
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile


TOOLS = Path(__file__).resolve().parent
PROJECT_ROOT = TOOLS.parent
DEFAULT_ICON = TOOLS / "app_icon"
APP_NAME = "Kirby Assist Workbench"
INSTALLED_ROOT = Path("/Applications")


def run_install_command(*command: str) -> None:
    if os.access(INSTALLED_ROOT, os.W_OK):
        subprocess.run(command, check=True)
    else:
        subprocess.run(["sudo", "-A", *command], check=True)


def remove_install_path(path: Path) -> None:
    run_install_command("/bin/rm", "-rf", str(path))


def resolve_icon(source: Path) -> Path:
    from PIL import Image

    if not source.is_dir():
        if not source.is_file():
            raise FileNotFoundError(f"App icon image or directory not found: {source}")
        return source
    for candidate in sorted(source.iterdir()):
        if not candidate.is_file():
            continue
        try:
            with Image.open(candidate) as image:
                image.load()
        except (OSError, ValueError):
            continue
        return candidate
    raise FileNotFoundError(f"No readable app icon image found in directory: {source}")


def make_icon(source: Path, destination: Path):
    from PIL import Image

    with Image.open(source) as original:
        image = original.convert("RGBA")
        image.thumbnail((1024, 1024), Image.Resampling.LANCZOS)
        canvas = Image.new("RGBA", (1024, 1024), (0, 0, 0, 0))
        canvas.alpha_composite(image, ((1024 - image.width) // 2, (1024 - image.height) // 2))
    destination.mkdir(parents=True, exist_ok=True)
    icon_path = destination / "AppIcon.icns"
    with tempfile.TemporaryDirectory(dir=destination) as temporary:
        image_set = Path(temporary) / "AppIcon.iconset"
        image_set.mkdir(parents=True)
        for size in (16, 32, 64, 128, 256, 512):
            for scale in (1, 2):
                pixels = size * scale
                name = f"icon_{size}x{size}" + (f"@{scale}x" if scale == 2 else "") + ".png"
                canvas.resize((pixels, pixels), Image.Resampling.LANCZOS).save(image_set / name)
        result = subprocess.run(["iconutil", "-c", "icns", str(image_set), "-o", str(icon_path)], check=True)
        if result.returncode == 0 and not icon_path.is_file():
            raise RuntimeError("iconutil succeeded without creating the ICNS icon")


def finalize_bundle(app: Path, icon: Path) -> None:
    plist_path = app / "Contents" / "Info.plist"
    with plist_path.open("rb") as manifest:
        plist = plistlib.load(manifest)
    plist["NSHighResolutionCapable"] = True
    with plist_path.open("wb") as manifest:
        plistlib.dump(plist, manifest)
    resources = app / "Contents" / "Resources"
    shutil.copy2(icon, resources / f"app_icon_source{icon.suffix.lower()}")
    subprocess.run(["/usr/bin/codesign", "--force", "--sign", "-", str(app)], check=True)
    verify_bundle(app)


def verify_bundle(app: Path) -> None:
    subprocess.run(["/usr/bin/codesign", "--verify", "--deep", "--strict", str(app)], check=True)
    with (app / "Contents" / "Info.plist").open("rb") as manifest:
        executable = plistlib.load(manifest)["CFBundleExecutable"]
    result = subprocess.run([str(app / "Contents" / "MacOS" / executable), "--help"],
                            capture_output=True, text=True, timeout=30, check=True)
    if "--project" not in result.stdout:
        raise RuntimeError(f"Built app did not pass its startup check: {app}")
    subprocess.run([str(app / "Contents" / "MacOS" / executable), "--check-terminal-bundle"],
                   capture_output=True, text=True, timeout=30, check=True)


def build(icon: Path, name: str):
    if sys.platform != "darwin":
        raise RuntimeError("The .app builder requires macOS (PyInstaller and iconutil)")
    try:
        importlib.import_module("PyInstaller")
        importlib.import_module("PIL")
        for module in ("textual", "textual_plotext", "rich_pixels"):
            importlib.import_module(module)
    except ImportError as error:
        preferred_python = TOOLS.parent / ".venv" / "bin" / "python"
        python = preferred_python if preferred_python.is_file() else Path(sys.executable)
        raise RuntimeError(f"Install macOS packaging requirements with: {python} -m pip install -r {TOOLS / 'requirements-macos.txt'}") from error
    icon = resolve_icon(icon)
    if shutil.which("cargo") is None:
        raise RuntimeError("Install the Rust/Cargo toolchain before building the Workbench app")

    if not name or name in {".", ".."} or Path(name).name != name:
        raise ValueError("App name must be a simple file name")
    build_root = TOOLS / "build" / "macos"
    build_root.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="workbench-", dir=build_root))
    build_dist = work / "dist"
    installed_app = INSTALLED_ROOT / f"{name}.app"
    install_stage = INSTALLED_ROOT / f".{name}.install-{os.getpid()}.app"
    install_backup = INSTALLED_ROOT / f".{name}.backup-{os.getpid()}.app"
    install_failed = INSTALLED_ROOT / f".{name}.failed-{os.getpid()}.app"
    install_complete = False
    try:
        icon_dir = work / "icon"
        make_icon(icon, icon_dir)
        datas = [
            (str(TOOLS / "workbench_gui.html"), "."),
            (str(TOOLS / "workbench_reminal_ui" / "app.tcss"), "workbench_reminal_ui"),
            (str(TOOLS / "workbench_ui" / "dashboard.css"), "workbench_ui"),
            (str(TOOLS / "workbench_ui" / "dashboard.js"), "workbench_ui"),
            (str(TOOLS / "workbench_ui" / "vendor" / "lucide.js"), "workbench_ui/vendor"),
            (str(TOOLS / "workbench_ui" / "vendor" / "chart.js"), "workbench_ui/vendor"),
        ]
        hidden_imports = [
            "tomllib", "fighter_editor", "sequence_sources", "sequence_transfer", "sequence_validation",
            "sync_sequences", "toml_compat", "upload_switch", "upload_all_switches",
            "workbench_history", "workbench_capture", "workbench_telemetry", "workbench_gui",
            "build_variants", "upload_sequences", "sync_sequences", "validate_fighter_data",
            "workbench_reminal_ui.launcher", "workbench_reminal_ui.app", "workbench_reminal_ui.backend",
            "workbench_reminal_ui.widgets", "workbench_reminal_ui.telemetry",
        ]
        build_python = sys.executable
        build_env = {**os.environ, "KIRBY_ASSIST_PROJECT_ROOT": str(PROJECT_ROOT), "KIRBY_ASSIST_FROZEN_ROOT": str(PROJECT_ROOT),
                     "PYINSTALLER_CONFIG_DIR": str(work / "cache")}
        build_architecture = platform.machine()
        command = [
            str(build_python), "-m", "PyInstaller", "--noconfirm", "--clean", "--windowed",
            "--name", name, "--icon", str(icon_dir / "AppIcon.icns"),
            "--distpath", str(build_dist), "--workpath", str(work / "pyinstaller"),
            "--specpath", str(work), "--osx-bundle-identifier", "com.kirbyassist.workbench",
            "--target-architecture", build_architecture,
        ]
        for source, target in datas:
            command.extend(["--add-data", f"{source}{os.pathsep}{target}"])
        for module in hidden_imports:
            command.extend(["--hidden-import", module])
        command.extend(["--collect-data", "PIL"])
        for package in ("textual", "textual_plotext", "rich_pixels", "plotext"):
            command.extend(["--collect-all", package])
        command.append(str(TOOLS / "workbench.py"))
        subprocess.run(command, cwd=TOOLS, env=build_env, check=True)

        app = build_dist / f"{name}.app"
        plist_path = app / "Contents" / "Info.plist"
        if not plist_path.is_file():
            raise RuntimeError(f"PyInstaller did not produce an app bundle at {app}")
        finalize_bundle(app, icon.resolve())

        if not (app / "Contents" / "Info.plist").is_file():
            raise RuntimeError(f"Built app bundle is incomplete: {app}")
        if not INSTALLED_ROOT.is_dir():
            raise RuntimeError(f"Installation directory does not exist: {INSTALLED_ROOT}")
        if os.access(INSTALLED_ROOT, os.W_OK):
            shutil.copytree(app, install_stage, symlinks=True)
        else:
            run_install_command("/usr/bin/ditto", str(app), str(install_stage))
        if not (install_stage / "Contents" / "Info.plist").is_file():
            raise RuntimeError(f"Staged app bundle is incomplete: {install_stage}")
        if installed_app.exists() or installed_app.is_symlink():
            run_install_command("/bin/mv", str(installed_app), str(install_backup))
        try:
            run_install_command("/bin/mv", str(install_stage), str(installed_app))
            verify_bundle(installed_app)
            install_complete = True
        except BaseException:
            if installed_app.exists() or installed_app.is_symlink():
                run_install_command("/bin/mv", str(installed_app), str(install_failed))
                remove_install_path(install_failed)
            if install_backup.exists() or install_backup.is_symlink():
                run_install_command("/bin/mv", str(install_backup), str(installed_app))
            raise
        if install_backup.exists() or install_backup.is_symlink():
            remove_install_path(install_backup)
        shutil.rmtree(work, ignore_errors=True)
        print(f"Installed: {installed_app}\nIcon source: {icon.resolve()}")
    finally:
        if not install_complete and (install_backup.exists() or install_backup.is_symlink()):
            if installed_app.exists() or installed_app.is_symlink():
                run_install_command("/bin/mv", str(installed_app), str(install_failed))
                remove_install_path(install_failed)
            run_install_command("/bin/mv", str(install_backup), str(installed_app))
        if install_stage.exists() or install_stage.is_symlink():
            remove_install_path(install_stage)
        shutil.rmtree(work, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--icon", type=Path, default=DEFAULT_ICON,
                        help=f"source image or directory; directories use the first readable image "
                             f"alphabetically, regardless of filename or extension (default: {DEFAULT_ICON})")
    parser.add_argument("--name", default=APP_NAME, help="installed app name")
    args = parser.parse_args()
    if not args.name or args.name in {".", ".."} or Path(args.name).name != args.name:
        parser.error("--name must be a simple file name")
    try:
        build(args.icon.expanduser().resolve(), args.name)
    except (FileNotFoundError, RuntimeError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Build failed: {error}\n")


if __name__ == "__main__":
    main()
