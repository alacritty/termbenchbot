#!/bin/sh

gnuplot_script="/home/undeadleech/vtebench/gnuplot_summary.sh"
output_file="/home/undeadleech/website/static/alacritty_master.svg"

# Pull the latest benchmark results.
git pull origin master --rebase

# Plot the last 10 results.
$gnuplot_script $(ls results/alacritty/master/*.dat | tail -n 10) $output_file
