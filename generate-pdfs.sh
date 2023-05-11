#!/usr/bin/env bash

set -e

if [ "$1" = "--clean" ]; then
    rm -rf out
    shift
fi

cargo run --release

pushd make-pdf

if [ ! -d "node_modules" ]; then
    pnpm i
fi
npx http-server ../out -p 8000 -s &
SERVER_PID=$!

npx tsx main.ts
kill $SERVER_PID

popd
