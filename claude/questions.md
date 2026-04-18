# Questions / Action items for guardian

## From issue #2 prototype work

**Q1. Android target**
The design says "web app and android app, one codebase preferred".
For the web side, Rust→WASM works. For Android, what is the intended approach?
Options:
- Progressive Web App (works in Android Chrome with home-screen install, near-zero extra work)
- Native Android app using the Rust core via the Android NDK + a thin Kotlin/Compose shell
- Something else (Tauri Mobile, etc.)?

*Which of these will start the fastest? Efficiency / latency is the key here.*
