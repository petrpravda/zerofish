importScripts('./zerofish_wasm.js');

const { newEngineContext, executeUciCommand, sendMessage, post_message_to_worker } = wasm_bindgen;

async function run() {
    await wasm_bindgen('./zerofish_wasm_bg.wasm');
    let engineContext = newEngineContext(self);
    console.log(`engineContext: ${engineContext}`);
    // const res1 = executeUciCommand('d', engineContext);
    executeUciCommand('d', engineContext);
    executeUciCommand('go depth 10', engineContext);
    //console.info(res1);
    self.postMessage({msg: 'started'});
    //sendMessage();
    //post_message_to_worker(self);

    // onmessage = function(e) {
    //     const {id, payload} = e.data;
    //     let result = undefined;
    //     let lines = payload.split('\n');
    //     lines.forEach(line => {
    //         const t0 = performance.now();
    //         result = executeUciCommand(line, engineContext);
    //         const t1 = performance.now();
    //         // console.log(`worker.js (${line}) -> ${result} (${(t1 - t0).toFixed(3)} ms)`);
    //         // console.log(`worker.js -> ??????? (${(t1 - t0).toFixed(3)} ms)`);
    //     });
    //     // console.log(`worker.js <- ${payload}`);
    //     postMessage({id, payload: result});
    // }
}

run().then(() => {
    postMessage({id: -1, err: undefined, payload: 'ready'});
});
