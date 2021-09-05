"""Utility for building a local copy of the Rust extention."""

import os
import shutil
import sys
import subprocess  # noqa: S404
from pathlib import Path


def main():
    """Build extension."""
    cwd = Path.cwd().resolve()

    # Build extention
    print("Building extention")

    # Unlcear why this is needed
    modified_env = os.environ.copy()
    modified_env["PYO3_PYTHON"] = sys.executable

    proc = subprocess.run(["cargo", "build", "--release"], env=modified_env)  # noqa: S603, S607
    if proc.returncode != 0:
        print("Build failed")
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
