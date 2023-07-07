#!/usr/bin/env bash

set -euo pipefail

# Uncomment following line to see executed commands
# set -x

rm -rf ./data/*

for i in $(seq 1 45);
do
    curl -v "https://api.icfpcontest.com/problem?problem_id=${i}" | jq -r '.Success' > "data/problem-${i}.json"
done
