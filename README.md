# Nintendrust

Nintendrust is a Rust-based emulator for the NES. It implements the full 6502 official instruction set, and a scanline-accurate PPU. It is portable, supporting both desktop environments and the web.

## Versions

On this repository there are two branches. The `main` branch contains the code for the web version of the emulator, and the `native` branch has the desktop version.

### Running the web version

The program can be found running at https://nintendrust.hawksley.dev/

Alternatively, you can run the program locally, or deploy to a hosting site like GitHub Pages.

Ensure you have the Cargo package manager installed. If you are missing it, you can download it through [rustup](https://rustup.rs/).

Then, install the `wasm-pack` crate.

```shell
cargo install wasm-pack
```

Clone the repository locally.

```shell
git clone https://github.com/ethanhawksley/nintendrust
cd nintendrust
```

Compile the Rust code to WASM.

```shell
wasm-pack build --target web
```

Finally, use a http server to host the `index.html` file. I recommend `serve`.

```shell
npx serve .
```

The program is now available to visit in your browser!

### Running the desktop version

Ensure you have the Cargo package manager installed. If you are missing it, you can download it through [rustup](https://rustup.rs/).

Clone the repository locally.

```shell
git clone https://github.com/ethanhawksley/nintendrust
cd nintendrust
git checkout native
```

Compile and run the program.

```shell
cargo run --release
```

The program shall now boot into the Zooming Secretary game that comes with the repository. To use a custom ROM, alter `src/main.rs` to select a custom path.

```rust
// ...
fn main() {
    let file_path = "Zooming_Secretary.nes";
    // ...
}
```

## Author

Made by [Ethan Hawksley](https://hawksley.dev)
