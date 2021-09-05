#!/bin/bash

set -u

BASE_DIR="$HOME/lightbox_debug"

# Download
$BASE_DIR/scripts/download_debug.sh
DL_STATUS="$?"

# Check status
if [ $DL_STATUS -eq 1 ]
then
  echo "Failed to download build"
  exit 1
fi

cd $BASE_DIR

killall -q lightbox
if cargo build --release --bin lightbox_client --features="lightbox_client"; then
    echo "Running"
    sudo RUST_BACKTRACE=1 RUST_LOG=debug "$HOME/cargo_target/release/lightbox_client" "$@"
else
    echo "Build failed"
fi
