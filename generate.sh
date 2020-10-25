#!/bin/bash
set -e

cargo run --release -- $1 | ffmpeg -f rawvideo -pix_fmt rgb24 -r 24 -s:v 800x600 -i - -c:v h264 -crf 0 -pix_fmt yuv420p out.mp4
