const { newEngineContext, executeUciCommand } = wasm_bindgen;

async function run() {
    await wasm_bindgen('./zerofish.wasm');
    const engineContext = newEngineContext(self);
    onmessage = function (e) {
        executeUciCommand(e.data, engineContext);
    }
}

run().then(() => {});
