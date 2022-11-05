# Bitcoin Blockstats Index and Display

1. Modify the `DATABASE_URL` and other vars in `.env` to point to your chosen database & bitcoin node
example .env file:
  HOST=127.0.0.1
  PORT=8000
  USERNAME=<btcnodeusername>
  PASSWORD=<btcrpcpassword>
  URL=<nodeurl:port>
  DATABASE_URL=postgres://postgres:postgres@localhost/db_name
  INDEX_FROM=<startingblock>
  INTERVAL=1000
  RETRY_ATTEMPTS=10

2. Turn on the appropriate database feature for your chosen db in `Cargo.toml` (the `"sqlx-postgres",` line)

3. Execute `cargo run` to start the server

4. Visit [localhost:8000](http://localhost:8000) in browser
