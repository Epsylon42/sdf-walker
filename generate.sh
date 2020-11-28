#!/bin/bash
set -eu

width=$2
height=$3

if [ -z "$4" ]; then
    fps=30
else
    fps=$4
fi

cargo run --release -- $1 render --width $width --height $height --fps $fps | ffmpeg -f rawvideo -pix_fmt rgb24 -r $fps -s:v "$width"x"$height" -i - -c:v h264 -crf 0 -pix_fmt yuv420p out.mp4
