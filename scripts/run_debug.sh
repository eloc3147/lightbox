#!/bin/bash

set -u

./download_debug.sh

# Check status
if [ $? -ne 0 ] ; then
  echo "Failed to download build"
  exit 1
fi

killall -q lightbox
cd $HOME/lightbox/debug
cargo clean -p lightbox
if cargo build; then
    echo "Running"
    sudo RUST_BACKTRACE=full target/debug/lightbox
else
    echo "Build failed"
fi
