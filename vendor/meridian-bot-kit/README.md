# Meridian bot kit

A starting point for a Meridian bot in Rust.

- `RULES.md`: the rules.
- `engine/`: the rules engine that runs https://constellation.blueshrimp.uk, a Rust crate with no
  dependencies. `engine/README.md` explains how to use it and how it works.
- `scout/`: Scout, the site's own bot, as a WebAssembly bot you can build and upload.

## Build Scout

You need Rust 1.85 or newer.

```sh
rustup target add wasm32-wasip1
cd scout
cargo build --release --target wasm32-wasip1
```

Upload `scout/target/wasm32-wasip1/release/scout.wasm` on the site: open **My bots**, choose
**Add a bot**, then **WebAssembly**. Your browser tests the file before it is uploaded and shows
anything the bot prints.

## Make it your own

- `scout/src/search.rs` is Scout's strategy. Replace it with yours, keeping `best_move` and `analyze`,
  or change `lib.rs` to call what you write instead.
- `scout/src/lib.rs` reads the site's requests, replays the position with the engine and writes the
  replies. It ends with the three functions the site calls; keep those as they are.
- Rename the crate in `scout/Cargo.toml`.

The bot API guide, https://constellation.blueshrimp.uk/bot-api.md, describes the requests, the replies and
the limits, and how to connect a bot written in any other language.
