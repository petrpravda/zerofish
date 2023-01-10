#!/bin/bash

cat pkg/zerofish_wasm.js inlined.worker.js > pkg/zerofish.js
cp pkg/zerofish_wasm_bg.wasm pkg/zerofish.wasm
cp pkg/zerofish.wasm pkg/zerofish.js web/public
