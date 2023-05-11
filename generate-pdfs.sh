#!/usr/bin/env bash

set -e

OUTDIR=$(cargo run --release -q -- $@ --print-out-dir)

pushd make-pdf

if [ ! -d "node_modules" ]; then
    pnpm i
fi
npx http-server "../$OUTDIR" -p 8000 -s &
SERVER_PID=$!

npx tsx main.ts
kill $SERVER_PID

popd
