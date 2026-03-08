# MicroCapital

Online viewer for Capital Essentials XML files.

The app is published at **https://shcherbakaite.github.io/microcapital/**

# License

Copyright (C) 2025 Vladislav Shcherbakov

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

---

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- For web: `rustup target add wasm32-unknown-unknown`
- For web build: [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

## Run native

```bash
cargo run
```

## Build for web (WASM)

Build the library only (web entry point is in `lib.rs`):

```bash
cargo build --target wasm32-unknown-unknown --lib
```

To get the JavaScript glue and a ready-to-serve `pkg/` folder, install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) and run:

```bash
wasm-pack build --target web --lib
```

Then serve the project directory (e.g. `python -m http.server 8080` or `npx serve`) and open `http://localhost:8080` in a browser. Load `index.html` (it loads `pkg/microcapital.js`).

## Deploy to GitHub Pages

The app is published as a project page at **https://shcherbakaite.github.io/microcapital/**.

1. In the repo: **Settings → Pages → Build and deployment**, set **Source** to **GitHub Actions**.
2. Push to `master` to trigger deployment. The workflow builds with Trunk (configured for the `/microcapital/` base path) and deploys to GitHub Pages.

## Project layout

- `src/app.rs` – app state and egui UI (implement `eframe::App`)
- `src/lib.rs` – WASM entry point (`start()`)
- `src/main.rs` – native entry point
- `index.html` – canvas and script tag for the web build

## License

GNU General Public License v3.0 or later. See [LICENSE](LICENSE) for the full text.
