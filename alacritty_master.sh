#!/bin/sh

alacritty_path="/home/undeadleech/programming/rust/alacritty"
hashfile="./lasthash"

# Benchmark the latest Alacritty master.

if [ $EUID -ne 0 ]; then
    echo "Error: Must be run as root"
    exit 1
fi

# Update to the latest version of Alacritty.
git -C "$alacritty_path" pull origin master --rebase

# Check if there has been any new commit.
hash=$(git -C "$alacritty_path" rev-parse HEAD)
if [ "$(cat \"$hashfile\")" = "$hash" ]; then
    echo "No new commits to test"
    exit
fi
printf "$hash" > "$hashfile"

# Make sure we have the latest rust stable toolchain.
rustup update stable

# Build the latest version.
cargo build --release --manifest-path "$alacritty_path/Cargo.toml"

./bench.sh "$alacritty_path/target/release/alacritty" results/alacritty/master/

# Push changes to GitHub.
shorthash=$(git -C "$alacritty_path" rev-parse --short HEAD)
message=$(date +"Alacritty master ($shorthash) %Y-%m-%dT%H:%M:%SZ")
git add "$hashfile" results/alacritty/master
git commit -m "$message"
git push origin master
