Dracula
===

## Rust-based ETL

A Rust-based ETL pipeline that extracts application data from PostgreSQL and other sources, transforms it into Parquet format, and loads it into AWS Athena for efficient querying.

Rust Version
===

rustc 1.69.0 (84c898d65 2023-04-16)

Setup
===

```bash
# install rust and tooling! change tool to nightly
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.69.0
source $HOME/.cargo/env
rustc --version
# install the database tool with support for postgres (you need libpq on your system) 
cargo install diesel_cli --no-default-features --features "postgres"
```

```bash
brew install libpq
echo 'export PATH="/opt/homebrew/opt/libpq/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```
