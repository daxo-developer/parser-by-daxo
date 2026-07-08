const fs = require('fs');

let alloc_ptr = 2000;

const importObject = {
    env: {
        print: (value) => console.log(`[WASM Print]: ${value}`),
        read_file: (name_ptr) => {
            const memory = new Uint8Array(global.instance.exports.memory.buffer);
            let name = "";
            let i = name_ptr;
            while (memory[i] !== 0) {
                name += String.fromCharCode(memory[i]);
                i++;
            }
            try {
                const content = fs.readFileSync(name, 'utf8');
                const offset = 1024;
                for (let j = 0; j < content.length; j++) {
                    memory[offset + j] = content.charCodeAt(j);
                }
                memory[offset + content.length] = 0;
                return offset;
            } catch (e) {
                return 0;
            }
        },
        write_file: (name_ptr, data_ptr, data_len) => {
            const memory = new Uint8Array(global.instance.exports.memory.buffer);
            let name = "";
            let i = name_ptr;
            while (memory[i] !== 0) {
                name += String.fromCharCode(memory[i]);
                i++;
            }
            const bytes = memory.slice(data_ptr, data_ptr + data_len);
            try {
                fs.writeFileSync(name, bytes);
                return 1;
            } catch (e) {
                return 0;
            }
        },
        alloc: (size) => {
            let res = alloc_ptr;
            alloc_ptr += size;
            return res;
        }
    }
};

async function run() {
    try {
        const wasmBuffer = fs.readFileSync('output.wasm');
        const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);
        global.instance = instance;
        console.log("=== Initializing Generated WebAssembly Module ===");
        instance.exports.main();
        console.log("=== Execution Completed Successfully ===");
    } catch (error) {
        console.error(`Runtime Error: ${error.message}`);
        process.exit(1);
    }
}

run();
