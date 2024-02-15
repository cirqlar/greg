#!/usr/bin/env bash
cd ./frontend
npm run build
cd ..
cargo build --release
