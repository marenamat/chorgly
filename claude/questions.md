# Questions / Action items for guardian

## Local dev env: missing system packages

`cargo check` requires `libssl-dev`, `cmake`, and `perl` (for the `git2` crate
which links against libgit2 + OpenSSL). These are already installed on CI via
the `apt-get install` step in `.github/workflows/build.yml`, but they are
missing locally.

Please install them:

```
sudo apt-get install -y pkg-config libssl-dev cmake perl
```

The `rustup`/`cargo` PATH issue was fixed — Rust itself is available. Only
the C-level build dependencies are still missing locally.
