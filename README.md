# sdf-walker

Tool for creating scenes using signed distance fields.

Run with `cargo run --release -- <path-to-scene-file> interactive [ -c ]`

Control camera with mouse and `WASDQE` keys. `escape` to exit. If the scene is animated, use `space` to toggle pause, `+` and `-` to rewind time, and `r` to reset time.

`-c` flag disables camera controls and uses camera descriptions in a scene file to move it.

If you have ffmpeg installed, you can also render a video with `./generate.sh <path-to-scene-file> <width> <height>`. It will create a file called `out.mp4`

There's no documentation for the scene language. Sorry.  
Considering this fact, using this tool is likely somewhere between "kind of a pain" to "literally impossible" for anyone who hasn't made it. You can run [examples](examples) or look at [screenshots](screenshots) though. They are very pretty, I promise.
