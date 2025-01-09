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
rustversion=$(sudo -u $user rustc -V)
version=$(echo "$($alacritty_path/target/release/alacritty --version) - $rustversion)")
lastversion=$(cat "$versionfile")
if [ "$lastversion" = "$version" ]; then
    echo "No new commits to test"
    exit
fi
printf "$version" | sudo -u $user tee "$versionfile"

./bench.sh "$alacritty_path/target/release/alacritty" "$output_directory"

# Update the plots.
mkdir -p "$output_directory/charts/"
max_cols=0
latest_ten=""
for file in $(ls results/alacritty/master/*.dat | tail -n 10); do
    # Get number of columns in the data file.
    cols=$(head -n 1 "$file" | awk '{print NF}')

    # Clear all files if we find one with more columns.
    if [ $cols -gt $max_cols ]; then
        max_cols=$cols
        latest_ten=""
    fi

    # With matching columns, add this file to the list.
    if [ $cols -eq $max_cols ]; then
        latest_ten=$(printf "$latest_ten\n$file")
    fi
done
"$vtebench_path/gnuplot/summary.sh" $(echo "$latest_ten") "$output_directory/charts/summary.svg"
"$vtebench_path/gnuplot/detailed.sh" $(echo "$latest_ten" | tail -n 3) "$output_directory/charts/"

# Push changes to GitHub.
shorthash=$(sudo -u $user git -C "$alacritty_path" rev-parse --short HEAD)
message=$(echo "Add results for Alacritty master ($shorthash)")
sudo -u $user git add "$versionfile" results/alacritty/master
sudo -u $user git commit -m "$message"
sudo -u $user git push origin master
