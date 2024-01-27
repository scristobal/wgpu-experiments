# wgpu-experiments

Messing around with wgpu 🦀🔺

## Instructions

This repo uses Trunk to target the browser, checkout the list of commands: <https://trunkrs.dev/commands/>

The default target is defined in `/.cargo/config.toml` as `wasm32-unknown-unknown` so if you want to build for your platform you must use `--target`

One last thing! check the shaders with `cargo wgsl`

## Manual build

In most cases using Trunk would be better, building the project w/o Trunk, requires WASM to be loaded manually:

```html
 <script type="module">
    import init from "./bin/wgpu_intro.js";
    init().then( () => console.log( "WASM Loaded" ) );
 </script>
```

then build specifying the target:

```bash
cargo build --target=wasm32-unknown-unknown --release
```

create the bindings:

```bash
wasm-bindgen ./target/wasm32-unknown-unknown/release/wgpu_intro.wasm --out-dir ./bin --target web
```

serve the files with you favorite localserver, eg. [cargo-server](https://crates.io/crates/cargo-server)

```bash
cargo server --open
```
