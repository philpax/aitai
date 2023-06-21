#!/bin/sh
set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <src> <dst>"
    exit 1
fi

src=$1
dst=$2

zstd -d --stdout $1 | tail -n 10000 > $dst