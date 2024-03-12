# syntax=docker/dockerfile:1
FROM rustlang/rust:nightly-slim

WORKDIR .

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos

COPY . .

RUN cargo leptos build -r

# Required
ENV LN_SETTINGS_FILE="/data/settings.toml"

# All available env variables
# ENV LN_DATA_DIR="/data/notes"
# ENV LN_DB_HOST="0.0.0.0:3000"
# ENV LN_DB_DATABASE="lazy_notes"
# ENV LN_DB_NAMESPACE="lazy_notes"
# ENV LN_DB_USERNAME="username"
# ENV LN_DB_PASSWORD="password"

EXPOSE 3000

CMD [ "cargo", "leptos", "serve", "--release" ]
