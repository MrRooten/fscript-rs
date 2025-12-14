#!/usr/bin/env sh

set -e

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <test_name>"
    dir="./test_script/bench/"
    for f in "$dir"/*; do
        [ -f "$f" ] || continue
        name="${f##*/}"
        echo "${name%.*}"
    done | sort -u
    exit 1
fi
cargo build --release

FS_FILE=./test_script/bench/"$1".fs
PYTHON_FILE=./test_script/bench/"$1".py

if [ ! -f "$FS_FILE" ]; then
    echo "FS File not existed: $FS_FILE"
    exit 1
fi

if [ ! -f "$PYTHON_FILE" ]; then
    echo "Python File not existed: $PYTHON_FILE"
    exit 1
fi

echo "FS:   $FS_FILE"
echo "Python: $PYTHON_FILE"
echo

hyperfine \
    --warmup 3 \
    "./target/release/fscript-rs $FS_FILE" \
    "python3 $PYTHON_FILE"
#!/usr/bin/sh

