#!/bin/sh
set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <src> <dst>"
    exit 1
fi

src=$1
dst=$2

QUERY=' select(
            .is_self and
            (.num_comments > 0) and
            (.score > 500) and
            (.link_flair_text != null) and
            (.name != null) and
            (.removed_by == null) and
            (.selftext != "[removed]") and
            (.selftext != "[deleted]") and
            (.link_flair_text | IN("Not the A-hole", "No A-holes here", "Everyone Sucks", "Asshole")) and
            (.sticked | not)
        ) | {
            title,
            text: .selftext,
            verdict: .link_flair_text,
            name,
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