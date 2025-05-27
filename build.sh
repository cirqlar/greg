#!/usr/bin/env bash
cd ./frontend
npm install
npm run build
cd ..
cargo build --release --features mail,scheduler
