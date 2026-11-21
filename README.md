<div align="center">

# Calculator

**Decimal arithmetic and history**

[![CI](https://img.shields.io/github/actions/workflow/status/PrimitiveOS/kindle-primitive-calculator/ci.yml?branch=main&label=CI)](https://github.com/PrimitiveOS/kindle-primitive-calculator/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/PrimitiveOS/kindle-primitive-calculator?label=release)](https://github.com/PrimitiveOS/kindle-primitive-calculator/releases)
[![Tag](https://img.shields.io/github/v/tag/PrimitiveOS/kindle-primitive-calculator?label=tag)](https://github.com/PrimitiveOS/kindle-primitive-calculator/tags)
[![Licence: PolyForm Noncommercial 1.0.0](https://img.shields.io/badge/licence-PolyForm%20Noncommercial%201.0.0-blue)](LICENSE)
[![Stars](https://img.shields.io/github/stars/PrimitiveOS/kindle-primitive-calculator?style=social)](https://github.com/PrimitiveOS/kindle-primitive-calculator/stargazers)
[![Store downloads](https://img.shields.io/endpoint?url=https%3A%2F%2Fprimitiveos.github.io%2Fkindle-primitive-catalog%2Fv1%2Fbadges%2Fprimitive-calculator.json)](https://primitiveos.github.io/kindle-primitive-catalog/v1/index.json)

[**Download**](https://github.com/PrimitiveOS/kindle-primitive-calculator/releases/latest) · [**Install guide**](INSTALL.md) · [**Changelog**](CHANGELOG.md)

</div>

---

Use large keys for addition, subtraction, multiplication and division. Browse calculation history and copy the current result to a text file.

Open Tools to convert common units, see steps for simplifying fractions or solving linear equations, and plot lines or quadratics with a table of values.

## Quick start

After setting up PrimitiveOS, open **Store** from Primitive Launcher. Choose **Calculator** under **Apps**,
tap **Get**, and open it from the Launcher grid. The [install guide](INSTALL.md) also covers the standalone USB package.

## Screenshots

<table>
  <tr>
    <td align="center"><img src="docs/images/calculator.png" alt="Calculator display and large e-ink keypad" width="260"><br><sub>The calculator keypad</sub></td>
    <td align="center"><img src="docs/images/calculator-history.png" alt="History showing 0.1 plus 0.2 with the result 0.3" width="260"><br><sub>Calculation history</sub></td>
    <td align="center"><img src="docs/images/calculator-copy-dark.png" alt="Calculator in dark mode after copying the result to a text file" width="260"><br><sub>Copy result in dark mode</sub></td>
  </tr>
</table>

<sub>Desktop simulation at 1264 × 1680 with sample calculations.</sub>

## Highlights

- Decimal arithmetic supports up to 18 digits and 12 decimal places. Results within those limits are exact.
- Nonterminating division is rounded and labelled. An operation other than division displays a message when the
  result requires more precision than the calculator supports.
- **History** keeps earlier calculations. **Copy result** writes `/mnt/us/calculator/result.txt`.

## Develop locally

Keep `doannt-kindle-tools` and `doannt-kindle-launcher` next to this checkout. The desktop command writes sample PNGs
to `build/previews`. The local `app.json` contains draft listing copy.

```sh
cargo test --offline --locked --workspace
cargo run --offline --locked -p calculator-desktop -- build/previews
```

## Status

| Device | Status |
|---|---|
| Kindle Oasis 3 (1264 × 1680, firmware 5.18.2) | Tested on hardware |

**Current version: 0.1.4.** Changes are in [CHANGELOG.md](CHANGELOG.md).

## Downloads

The badge at the top shows the number of Store downloads. It counts GitHub downloads of this app's release packages, across all versions and architectures, and is refreshed daily from the [Store catalog](https://primitiveos.github.io/kindle-primitive-catalog/v1/index.json).

## Documentation

[Install guide](INSTALL.md) · [Changelog](CHANGELOG.md) · [Device notes](docs/device-notes.md)

## Licence

Copyright (C) 2026 Doan Nguyen.

App code is licensed under PolyForm Noncommercial 1.0.0. [Code license](LICENSE)

Original documentation and screenshots are licensed under Creative Commons Attribution-NonCommercial-ShareAlike 4.0
International (CC BY-NC-SA 4.0). [Documentation and image license](LICENSE-DOCS)
