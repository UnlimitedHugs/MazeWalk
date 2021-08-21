#!/bin/bash
PAGES_DIR="./pages"
if [ $# -eq 0 ]; then
    echo "Commit message argument is required"
	exit 1
fi
if ! [ -d "$PAGES_DIR" ]; then
	echo "Clone pages repo..."
	git clone https://github.com/UnlimitedHugs/maze-walk $PAGES_DIR
fi
echo "Copy files..."
rm -r $PAGES_DIR/assets
cp -r ./assets $PAGES_DIR/assets
cp -r ./pkg/gl.js ./pkg/index.html ./pkg/maze_walk.wasm $PAGES_DIR
if [ -x "$(command -v wasm-opt)" ]; then
	echo "Optimizing wasm..."
	wasm-opt $PAGES_DIR/maze_walk.wasm -o $PAGES_DIR/maze_walk.wasm -O4
else 
	echo wasm-opt not found, skipping
fi
cd $PAGES_DIR
git add . &&
git commit -am $1
read -p "Push changes to remote? (y/n) " -n 1 -r
echo 
if [[ $REPLY =~ ^[Yy]$ ]]
then
    git push
fi
