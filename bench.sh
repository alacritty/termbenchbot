#!/bin/sh

vtebench_path="/home/undeadleech/programming/rust/vtebench"

if [ $# -lt 2 ]; then
    echo "Usage: bench.sh <TERMINAL> <OUTPUT_DIRECTORY>"
    exit 1
fi

if [ $EUID -ne 0 ]; then
    echo "Error: Must be run as root"
    exit 2
fi

term=$(command -v "$1")
if ! [ -x "$term" ]; then
    echo "Error: Command not found: $1"
    exit 3
fi

# Make sure the latest version of vtebench is installed.
git -C "$vtebench_path" pull origin master --rebase

cargo build --release --manifest-path "$vtebench_path/Cargo.toml"

vtebench="$vtebench_path/target/release/vtebench"
bench_dir="$vtebench_path/benchmarks"

# Create output directory if it doesn't exist already.
output_dir="$2"
mkdir -p "$output_dir"
output_dir=$(realpath "$output_dir")

version=$("$term" --version 2> /dev/null || "$term")
output_file=$(date +"$output_dir/${version}_%Y-%m-%dT%H:%M:%SZ.dat" | tr " " "_")

# Setup environment to improve benchmark consistency.
echo "0" > /proc/sys/kernel/randomize_va_space
systemctl stop autopoweroff

XINITRC=/dev/null xinit "$term" -e "$vtebench" -s -b "$bench_dir" --dat "$output_file" --warmup 3 --max-secs 60

# Recover environment setup.
echo "2" > /proc/sys/kernel/randomize_va_space
systemctl start autopoweroff
