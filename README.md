# Primitive Calculator

The calculator uses decimal arithmetic with up to 18 digits and 12 decimal places. Results are exact where representable. Nonterminating division is rounded and labelled, and over-precise nondivision results display a message. Copy result writes `/mnt/us/calculator/result.txt` for USB or another local app.

Run `cargo test --offline --workspace` and `cargo run --offline -p calculator-desktop` with
`doannt-kindle-tools` and `doannt-kindle-launcher` next to this checkout. The desktop target
writes 1264 × 1680 grayscale previews in `build/previews/`.

See [installation instructions](INSTALL.md). The local `app.json` is a listing draft. A
public listing requires approved artwork and an asset from a published release.

Source is PolyForm Noncommercial 1.0.0. Documentation is CC BY-NC-SA 4.0.
