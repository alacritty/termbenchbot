#!/bin/sh

user="perfbot"
versionfile="./lastversion"
vtebench_path="/home/$user/vtebench"
alacritty_path="/home/$user/alacritty"
output_directory="results/alacritty/master"

# Benchmark the latest Alacritty master.

if [ $EUID -ne 0 ]; then
    echo "Error: Must be run as root"
    exit 11
fi

if ! id "$user" &> /dev/null; then
    echo "Error: User \"$user\" does not exist"
    exit 12
fi

# Clone Alacritty if it does not exist yet.
if ! [ -d "$alacritty_path" ]; then
    sudo -u $user git clone https://github.com/alacritty/alacritty "$alacritty_path"
fi

# Update to the latest version of Alacritty.
sudo -u $user git -C "$alacritty_path" pull origin master --rebase

# Build the latest version.
sudo -u $user cargo build --release --manifest-path "$alacritty_path/Cargo.toml"

# Check if there has been any new changes.
version=$("$alacritty_path/target/release/alacritty" --version)
lastversion=$(cat "$versionfile")
if [ "$lastversion" = "$version" ]; then
    echo "No new commits to test"
    exit
fi
printf "$version" | sudo -u $user tee "$versionfile"

./bench.sh "$alacritty_path/target/release/alacritty" "$output_directory"

# Update the plot for the last 10 results.
"$vtebench_path/gnuplot/summary.sh" $(ls results/alacritty/master/*.dat | tail -n 10) "$output_directory/summary.svg"

# Push changes to GitHub.
shorthash=$(git -C "$alacritty_path" rev-parse --short HEAD)
message=$(date +"Add results for Alacritty master ($shorthash)")
sudo -u $user git add "$versionfile" results/alacritty/master
sudo -u $user git commit -m "$message"
sudo -u $user git push origin master
