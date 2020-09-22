#!/bin/sh

user="perfbot"
hashfile="./lasthash"
alacritty_path="/home/$user/alacritty"
output_directory="results/alacritty/master/"

# Benchmark the latest Alacritty master.

if [ $EUID -ne 0 ]; then
    echo "Error: Must be run as root"
    exit 11
fi

if ! id "$user" &> /dev/null; then
    echo "Error: User \"$user\" does not exist"
    exit 12
fi

# Run as much as possible without root permissons.
# Clone Alacritty if it does not exist yet.
if ! [ -d "$alacritty_path" ]; then
    sudo -u $user git clone https://github.com/alacritty/alacritty "$alacritty_path"
fi

# Update to the latest version of Alacritty.
sudo -u $user git -C "$alacritty_path" pull origin master --rebase

# Check if there has been any new commit.
hash=$(git -C "$alacritty_path" rev-parse HEAD)
lasthash=$(cat "$hashfile")
if [ "$lasthash" = "$hash" ]; then
    echo "No new commits to test"
    exit
fi
printf "$hash" | sudo -u $user tee "$hashfile"

# Build the latest version.
sudo -u $user cargo build --release --manifest-path "$alacritty_path/Cargo.toml"

./bench.sh "$alacritty_path/target/release/alacritty" "$output_directory"

# Push changes to GitHub.
shorthash=$(git -C "$alacritty_path" rev-parse --short HEAD)
message=$(date +"Alacritty master ($shorthash) %Y-%m-%dT%H:%M:%SZ")
sudo -u $user git add "$hashfile" results/alacritty/master
sudo -u $user git commit -m "$message"
sudo -u $user git push origin master
