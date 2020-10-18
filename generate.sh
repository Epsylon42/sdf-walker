#!/bin/bash
set -eu

cargo run --release --features generated -- "$1" | ffmpeg -f rawvideo -pix_fmt rgb24 -r 24 -s:v 800x600 -i - -c:v mpeg4 out.mp4
