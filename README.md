Dracula
===

Converts application data from postgres in to parquet! Whoa, spooky!

Rust Version
===

    rustup default nightly

Setup
===

```bash
# install rust and tooling! change tool to nightly
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2021-12-09
# install the database tool with support for postgres (you need libpq on your system) 
cargo install diesel_cli --no-default-features --features "postgres"
```

```bash
brew install libpq
echo 'export PATH="/opt/homebrew/opt/libpq/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```
