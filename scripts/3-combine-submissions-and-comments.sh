#!/bin/sh
set -e

if [ $# -ne 3 ]; then
    echo "Usage: $0 <submissions_src> <comments_src> <dst>"
    exit 1
fi

submissions_src=$1
comments_src=$2
dst=$3

QUERY=' {
            title,
            text,
            comments: $comments[.name],
            verdict
        }
        | select(.comments != null)'

jq -c "$QUERY" --argfile comments $comments_src $submissions_src > $dst