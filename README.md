# ai2048

This repository contains a modified 2048 game

Unlike the original, this version adds an AI written in Rust and compiled into WASM.

The repository also contains a commandline runner for the AI, mostly for debugging and benchmarking purposes.

## Building

You'll need [Rust](https://www.rust-lang.org/) in order to build the AI. Get it [here](https://rustup.rs/).

To compile it to WASM, you'll need [wasm-pack](https://github.com/rustwasm/wasm-pack). Installation instructions [here](https://rustwasm.github.io/wasm-pack/installer).

Finally, to build the website, you'll need [NPM](https://www.npmjs.com/).

After all the prerequisites are installed:

```bash
chmod +x build.sh && ./build.sh
```

## Acknowledgements

The original is written by Gabriele Cirulli. You can find it [here](https://github.com/gabrielecirulli/2048).

The AI uses techniques and heuristics inspired by this AI implementation:

https://github.com/nneonneo/2048-ai

This is a very good AI written in C++.
