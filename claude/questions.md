# Questions / Action items for guardian

## CI: Build & Test still failing (cargo check + wasm-pack)

Both the native `cargo check --workspace --exclude chorgly-frontend` and
`wasm-pack build` steps fail on every CI run. The cmake fix in `fe87cea` did
not resolve it. Code review has found no obvious compilation errors.

I cannot access raw CI logs (403 without admin rights) and the local
environment has no Rust toolchain, so I cannot reproduce the error.

**Please either:**
a) Install `rustup` + Rust stable so I can run `cargo check` locally and fix
   whatever is broken; or
b) Paste the relevant compiler error lines from the failed CI run into a
   comment on issue #3, and I will fix them on the next run.

*fixed $PATH so that rustup and cargo are available now*
