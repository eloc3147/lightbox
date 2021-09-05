"""Utility for building a local copy of the Rust extention."""

import shutil
import subprocess  # noqa: S404
from pathlib import Path


def main():
    """Build extension."""
    cwd = Path.cwd().resolve()

    # Build extention
    print("Building extention")
    proc = subprocess.run(["cargo", "build", "--release"])  # noqa: S603, S607
    if proc.returncode != 0:
        print("Build failed")
        print("If you're seeing linking errors, you probably don't have libftdi1 installed")
        print("Check the build instructions, and failing that, contact Kinnon for assistance")
        return

    src_path = cwd / "build" / "release" / "processing_test.dll"
    dst_path = cwd / "processing_test.pyd"

    if not src_path.exists():
        print("Error: Build artifact not found")
        return

    shutil.copyfile(src_path, dst_path)
    print("Built")


if __name__ == "__main__":
    main()
