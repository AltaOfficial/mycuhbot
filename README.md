# Mycuh Bot

clone with submodules you have access to private data 
```bash
git clone --recurse-submodules
```

setup .env first, must add path to sqlite.db file
see .env.example

run 
```bash
sqlx migrate run
```
to setup database

run
```bash
cargo run
```
to run bot

or

```bash
cargo build --release
./target/release/mycuhbot
```
to run binary
