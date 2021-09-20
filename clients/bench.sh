#!/bin/bash

# API Client for benchmarking available jobs.

if [[ $# -lt 1 ]]; then
    echo "Usage: bench.sh <JOBS_URL>"
    exit 1
fi

if [ $EUID -ne 0 ]; then
    echo "Error: Must be run as root"
    exit 11
fi

user="perfbot"
bench_dir="/tmp/benchdir"
gnuplot_script="/home/$user/vtebench/gnuplot/summary.sh"
termbenchbot_dir="/home/$user/termbenchbot"
bench_script="$termbenchbot_dir/bench.sh"
jobs_url="$1"

while true; do
    # Get the next available job.
    job=$(curl "$jobs_url" 2> /dev/null | jq -c '.[]' | head -n 1)

    if [ -n "$job" ]; then
        repository=$(echo "$job" | jq -r '.repository')
        sha=$(echo "$job" | jq -r '.hash')
        id=$(echo "$job" | jq -r '.id')

        # Mark job as in-progress.
        curl -X PATCH "$jobs_url/$id"

        # Build the commit in question.
        rm -rf "$bench_dir"
        sudo -u $user git clone "https://github.com/$repository" "$bench_dir"
        cd "$bench_dir"
        sudo -u $user git fetch origin $sha
        sudo -u $user git checkout $sha
        sudo -u $user cargo build --release

        # Run benchmark and generate output SVG.
        iso_date=$(date +"%Y-%m-%dT%H:%M:%SZ")
        output_file="results/alacritty/perfbot/$iso_date-$sha.svg"
        "$bench_script" "./target/release/alacritty" "./"
        master=$(ls $termbenchbot_dir/results/alacritty/master/*.dat | tail -n 1)
        "$gnuplot_script" "$master" *.dat "$termbenchbot_dir/$output_file"

        # Push SVG to termbenchbot for long-time storage.
        cd "$termbenchbot_dir"
        sudo -u $user git add "$output_file"
        sudo -u $user git commit -m "Results for $repository#$sha"
        sudo -u $user git push origin master
        image_url="https://raw.githubusercontent.com/alacritty/termbenchbot/master/$output_file"

        # Upload the results.
        curl -X POST --data "{\"result\": \"![results]($image_url)\"}" "$jobs_url/$id"
    fi

    sleep 60
done
