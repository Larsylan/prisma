# Contributing to Prisma

Thanks for your interest! Bug reports, ideas, translations and pull requests are all welcome.

- **Questions and ideas**: open a [discussion](https://github.com/Larsylan/prisma/discussions).
- **Bugs**: open an [issue](https://github.com/Larsylan/prisma/issues/new/choose). Never paste your NPSSO token or Steam Web API key.
- **Good first contributions**: look for issues labelled [`good first issue`](https://github.com/Larsylan/prisma/labels/good%20first%20issue).

## Getting started

You need [Rust](https://rustup.rs) 1.87+ and [Node.js](https://nodejs.org) 22 (plus the Linux libraries listed in the [README](README.md#building-from-source)).

```sh
npm ci
npm run tauri dev   # full app, reloaded on every change
npm run demo        # interface alone in a browser, on a fictional library (?lang=en or ?lang=fr)
```

To try Steam detection without a Steam account, run the app on a fake Steam client:

```sh
cargo run -p prisma-core --features test-support --example steam_fixture -- /tmp/steam
PRISMA_STEAM_ROOT=/tmp/steam PRISMA_DATA_DIR=/tmp/prisma npm run tauri dev
cargo run -p prisma-core --features test-support --example steam_fixture -- /tmp/steam unlock
```

`PRISMA_DATA_DIR` keeps your real library and PSN account out of the way.

## Project layout

| Folder | Content |
|---|---|
| `crates/prisma-core` | Business logic with no UI: trophies, merging, Steam / PSN / Switch sources, persistence, chime |
| `src-tauri` | Tauri backend: services, sync, notifications, tray icon |
| `src` | Svelte 5 + TypeScript interface |
| `src/lib/i18n` | Interface texts, in French (`fr.ts`, the reference) and English (`en.ts`) |
| `scripts` | macOS install, smoke test run by CI, demo recording |

## Before opening a pull request

```sh
npm run check
npm test
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same checks on macOS, Windows and Linux, then builds the installers, installs them and launches the app on a fake Steam client.

A few conventions:
- Keep changes focused: one pull request per topic.
- Any new interface text goes in **both** dictionaries; TypeScript reports a missing key.
- Code comments are written in French and kept for what the code cannot say by itself. You may write yours in English.
- No personal data in commits: no Steam IDs, tokens, e-mail addresses or local paths.

## Translations

Adding a language means adding a dictionary next to `en.ts`, registering it in `src/lib/i18n/index.svelte.ts`, and adding the language to `Language` in `src-tauri/src/i18n.rs` (tray menu and test notification). The Nintendo Switch trophy catalog is currently in French only.

## License

By contributing, you agree that your contributions are released under the [MIT License](LICENSE).
