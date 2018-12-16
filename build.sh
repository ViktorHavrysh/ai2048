#!/bin/bash

# install prerquisites
curl https://sh.rustup.rs -sSf | sh -s -- -y
source ~/.cargo/env
curl -o- https://raw.githubusercontent.com/creationix/nvm/v0.33.11/install.sh | bash
source ~/.nvm/nvm.sh
nvm install v10.5
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- -f

# build website
mkdir -p target && ln -sf ../../target www/ai2048-wasm/target
cd www/ai2048-wasm
wasm-pack build
cd ..
npm install
npm run build
cd ..
