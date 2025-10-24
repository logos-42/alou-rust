// Simple build script to wrap WASM module
import { readFileSync, writeFileSync } from 'fs';
import { resolve } from 'path';

const wasmPath = resolve('./target/wasm32-unknown-unknown/release/alou_edge.wasm');
const wasmBytes = readFileSync(wasmPath);
const wasmBase64 = wasmBytes.toString('base64');

const wrapper = `
// Auto-generated WASM wrapper
const wasmModule = Uint8Array.from(atob('${wasmBase64}'), c => c.charCodeAt(0));

export default {
  async fetch(request, env, ctx) {
    const { instance } = await WebAssembly.instantiate(wasmModule, {
      env,
    });
    
    if (instance.exports.fetch) {
      return instance.exports.fetch(request, env, ctx);
    }
    
    return new Response('Worker loaded but no fetch handler found', { status: 500 });
  }
};
`;

writeFileSync('./build/worker.mjs', wrapper);
console.log('Build complete!');
