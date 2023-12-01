# Budgeting Utility
For now just exploring what I can do with a relational db + rust CLI for personal budgeting.

## Getting set up
1. Install rust
1. Install sqlx cli `cargo install sqlx`
1. Create db and apply migrations
```bash
export DATABASE_URL=sqlite://local.db
sqlx db create
sqlx migrate run
```
1. Run unit tests `cargo test`
1. Use the application via the CLI `cargo run`

