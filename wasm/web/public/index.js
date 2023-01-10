// const importObject = {};
//
// WebAssembly.instantiateStreaming(fetch('zerofish_wasm_bg.wasm'), importObject)
//     .then(({instance}) => {
//             console.info(instance);
//         const newEngineContext = instance.exports.newEngineContext;
//         const executeUciCommand = instance.exports.executeUciCommand;
//         const engineContext = newEngineContext();
//         const uciCommand = 'uci';
//         const output = executeUciCommand(uciCommand, engineContext);
//         console.log(output);
//     });
//

if (typeof window !== "undefined") {
    const worker = new Worker('zerofish.js');
    //console.info(worker);
    worker.onmessage = function(event) {
        console.log(event.data);
    }
}
