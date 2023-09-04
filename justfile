##
# To learn how to do things manually, in most cases using Trunk would be better.
##

# builds the project w/o Trunk, requires WASM to be loaded manually (copy commented lines into index.html)
build:
    cargo build --target=wasm32-unknown-unknown --release
    wasm-bindgen ./target/wasm32-unknown-unknown/release/wgpu_intro.wasm --out-dir ./bin --target web

# the following snipppet needs to be included in `index.html`
#
#  <script type="module">
#     import init from "./bin/wgpu_intro.js";
#     init().then( () => console.log( "WASM Loaded" ) );
#  </script>

# launch devserver
serve:
    devserver --address localhost:3000


check:
    cargo wgsl
    cargo check
