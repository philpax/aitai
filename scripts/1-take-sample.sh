#!/bin/sh
set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <src> <dst>"
    exit 1
fi

src=$1
dst=$2

LIMIT=10000

if [[ $src == *.ndjson ]]; then
    echo "Extracting from ndjson..."
    tail -n $LIMIT $src > $dst
    echo "Done!"
elif [[ $src == *.zst ]]; then
    echo "Extracting from zst..."
    zstd -d --stdout $1 | tail -n $LIMIT > $dst
    echo "Done!"
else
    echo "Unknown file extension"
    exit 1
fi
