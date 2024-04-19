#!/bin/sh
set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <src> <dst>"
    exit 1
fi

src=$1
dst=$2

QUERY=' select(
            (.is_submitter | not) and 
            (.stickied | not) and 
            (.sticked | not) and
            (.score > 0) and 
            (.parent_id == .link_id) and
            (.author != "AutoModerator") and
            (.author != "Judgement_Bot_AITA") and
            (.body != "[deleted]") and
            (.body != "[removed]")
        ) | {
            body,
            link_id,
            score
        }'

if [[ $src == *.ndjson ]]; then
    echo "Extracting from ndjson..."
    jq -r -c "$QUERY" < $src > $dst
    echo "Done!"
elif [[ $src == *.zst ]]; then
    echo "Extracting from zst..."
    zstd -d --stdout $src | jq -r -c "$QUERY" > $dst
    echo "Done!"
else
    echo "Unknown file extension"
    exit 1
fi