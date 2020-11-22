#!/bin/bash

set -u

# Config
TIMEOUT=120
SERVER=$LIGHTBOX_FTP_IP
USER=$LIGHTBOX_FTP_USER
PROJECT="lightbox"


BASE_DIR="$HOME/lightbox"
DEBUG_DIR="$BASE_DIR/debug"
# Download
echo "Downloading $PROJECT (debug)"
mkdir -p $DEBUG_DIR
wget -q -nH -X target -X .git -X .vscode --show-progress --directory-prefix=$DEBUG_DIR --ftp-user=$USER -r -N -l inf ftp://$SERVER

# Check status
if [ $? -ne 0 ] ; then
  echo "Failed to download build"
  exit 1
fi
