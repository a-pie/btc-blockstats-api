# Bitcoin Blockstats Index and Display

1. Modify the `DATABASE_URL` and other vars in `.env` to point to your chosen database & bitcoin node

1. Turn on the appropriate database feature for your chosen db in `Cargo.toml` (the `"sqlx-postgres",` line)

1. Execute `cargo run` to start the server

1. Visit [localhost:8000](http://localhost:8000) in browser
