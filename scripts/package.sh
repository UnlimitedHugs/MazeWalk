#!/bin/bash
if ! [ -x "$(command -v 7z)" ]; then
	echo "7zip is required"
	exit 1
fi
cargo build --release --no-default-features
rm -f ~/Desktop/maze_walk.zip && 
7z a ~/Desktop/maze_walk.zip ./assets ./target/release/maze_walk.exe ./pkg/README.txt &&
echo "Archive added to desktop."