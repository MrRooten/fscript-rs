#!/bin/bash

set -e

PROGRAM="cargo test"

i=1
while true; do
    echo "Run #$i..."
    $PROGRAM
    status=$?
    if [ $status -ne 0 ]; then
        echo "❌ Program exited with non-zero status: $status"
        break
    fi
    i=$((i+1))
done

echo "🛑 Program crashed or exited with error after $i runs."
