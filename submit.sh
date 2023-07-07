#!/usr/bin/env bash

set -euo pipefail

# Uncomment following line to see executed commands
# set -x

PROBLEM_ID=$1
SOLUTION=$2
if [ -f $SOLUTION ]; then
    SOLUTION=$(cat $SOLUTION)
fi
JSON_SOLUTION=$(jq '@json' <<< ${SOLUTION})
curl -XPOST https://api.icfpcontest.com/submission -H "authorization: Bearer $(cat ./token)" -H 'content-type: application/json' -d "{\"problem_id\": ${PROBLEM_ID}, \"contents\": ${JSON_SOLUTION}}"
echo ""