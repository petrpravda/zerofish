const { newEngineContext, executeUciCommand } = wasm_bindgen;

async function run() {
    await wasm_bindgen('./zerofish.wasm');
    const buffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT);
    const sharedArray = new Int32Array(buffer);
    Atomics.store(sharedArray, 0, 0);

    const engineContext = newEngineContext(self, sharedArray);
    onmessage = function (e) {
        // console.info(`processing ${e.data}`);
        const cmd = e.data; //.startsWith('stop') ? 'stop' : e.data;
        if (cmd !== 'stop') {
            Atomics.store(sharedArray, 0, 0);
        }
        executeUciCommand(cmd, engineContext);
        // console.info(`finished processing ${cmd}`);
    }
    postMessage(buffer);
}

run().then(() => {});
