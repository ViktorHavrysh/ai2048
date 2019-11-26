#!/bin/bash

# install prerquisites
curl https://sh.rustup.rs -sSf | sh -s -- -y
source ~/.cargo/env
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.1/install.sh | bash
source ~/.nvm/nvm.sh
nvm install v12.13
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# build website
mkdir -p target && ln -sf ../target ai2048-wasm/target
cd www
npm install
npm run build
cd ..
