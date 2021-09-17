#!/bin/bash

# API client using WoL to automatically start other systems when pending jobs
# are available.

if [[ $# -lt 2 ]]; then
    echo "Usage: wol.sh <JOBS_URL> <MAC>"
    exit 1
fi

jobs_url="$1"
mac="$2"

while true; do
    # Check for new jobs.
    result=$(curl "$jobs_url" 2> /dev/null | grep "id" | wc -l)

    # Wake system if jobs are available.
    if [[ "$result" -gt 0 ]]; then
        wol "$mac"
    fi

    sleep 60
done
