#!/bin/sh
set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <src> <dst>"
    exit 1
fi

src=$1
dst=$2

QUERY=' 5 as $limit
        | group_by(.link_id)
        | map(select(length >= $limit))
        | map(sort_by(-.score)[:$limit])
        | map({(.[0].link_id): [.[] | .body]})
        | add
        '

if [[ $src == *.ndjson ]]; then
    echo "Extracting from ndjson..."
    jq -r -s "$QUERY" < $src > $dst
    echo "Done!"
else
    echo "Unknown file extension"
    exit 1
fi