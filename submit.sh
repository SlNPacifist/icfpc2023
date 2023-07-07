#!/usr/bin/env bash

set -euo pipefail

# Uncomment following line to see executed commands
# set -x

PROBLEM_ID=$1
SOLUTION_PATH=$2
TMP_PATH=$(mktemp)
jq "{\"problem_id\": ${PROBLEM_ID}, \"contents\": .|@json}" < $SOLUTION_PATH > $TMP_PATH
curl -XPOST https://api.icfpcontest.com/submission -H "authorization: Bearer $(cat ./token)" -H 'content-type: application/json' -d @"${TMP_PATH}"
echo ""