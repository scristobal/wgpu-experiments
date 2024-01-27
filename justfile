Use this file to do things manually, in most cases using Trunk would be better.
building the project w/o Trunk, requires WASM to be loaded manually:

 <script type="module">
    import init from "./bin/wgpu_intro.js";
    init().then( () => console.log( "WASM Loaded" ) );
 </script>

build:
    cargo build --target=wasm32-unknown-unknown --release
    wasm-bindgen ./target/wasm32-unknown-unknown/release/wgpu_intro.wasm --out-dir ./bin --target web

serve:
    devserver --address localhost:3000

