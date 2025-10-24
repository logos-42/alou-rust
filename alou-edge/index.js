// Import WASM module compiled with workers-rs
import * as wasm from './target/wasm32-unknown-unknown/release/alou_edge.wasm';

// workers-rs automatically exports the fetch handler
// Just re-export it
export default wasm;
