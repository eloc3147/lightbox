#!/bin/bash

set -u

# Config
TIMEOUT=120
SKIP_DIRS="-X *.git/ -X .vscode/ -X .cargo/ -X build/ -X render/"

SERVER=$LIGHTBOX_FTP_IP
USER=$LIGHTBOX_FTP_USER
BASE_DIR="$HOME/lightbox_debug"

# Download
echo "Downloading source"
mkdir -p "$BASE_DIR"

lftp -u "$USER," -e "mirror -c -e $SKIP_DIRS / /$BASE_DIR;exit" ftp://$SERVER

# Check status
if [ $? -ne 0 ] ; then
  echo "Failed to download build"
  exit 1
fi

chmod -R +x $BASE_DIR/scripts

exit 0
