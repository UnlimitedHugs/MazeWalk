#!/bin/bash
if ! command -v penguin &> /dev/null
then
	echo "Install dev server: cargo install penguin"
else	
	penguin serve ./pkg --mount assets:assets
fi
