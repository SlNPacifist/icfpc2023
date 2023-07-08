#!/usr/bin/env bash

set -euo pipefail

# Uncomment following line to see executed commands
# set -x

for i in $(seq 1 90);
do
    ./submit.sh "${i}" "solutions/problem-${i}.json"
done
