# Lazy Notes

Lazy Notes is a web frontend for your markdown notes.
Markdown files are lazily rendered to HTML when viewed allowing you to
instantly see any changes you make to your files locally.
Making use of Rust and the Leptos web framework, this project aims to be an
extremely fast and lightweight option for securely viewing your notes on
the web.

# Building

## Nix Users

The easiest way to build from source is to use the provided Nix flake.
If Nix is installed on your system, run the following command from the
project's root:

```
nix build .#server
```

### Docker

A derivation for building a Docker image is available in the flake.
Using Nix, not only are these images reproducible but they are also much
smaller than an equivalent build from a Dockerfile.
See the *Docker* section below for environment variables.

Run the following commands to build and load the image:

```
docker build .#dockerImage
docker image load -i result
```

## Non-Nix Users

`cargo-leptos` is used to build the project.
Assuming you have rust installed, run the following command:

```sh
cargo install --locked cargo-leptos
```

See the [cargo-leptos](https://github.com/leptos-rs/cargo-leptos/) repository
for more details.

<br>

From the project root, you can run the project by running:

```sh
cd server
cargo leptos --release serve
```

## Docker

If Nix is available on your system, it is recommended to use it to build the
Docker image as these images are both smaller and reproducible.

For non-Nix users, a Dockerfile is available to build this project for Docker.
From the project's root, run the following commands:

```
docker build -t lazy-notes server/
```

### Environment Variables

Settings are defined using a `settings.toml` file. By default, it will be
searched for in the current directory unless overwritten using the
`LN_SETTINGS_FILE` environment variable.

NOTE: Settings set in `settings.toml` will be overwritten by their
corresponding environment variable.

| Variables                 | Description                                            |
| ------------------------- | ------------------------------------------------------ |
| `LN_SETTINGS_FILE`        | Location of your settings file                         |
| `LN_DATA_DIR`             | Location of your data directory                        |
| `LN_ENABLE_REGISTRATION`  | Flag to enable/disable registration (e.g. true/false)  |
| `LN_DB_HOST`              | Address of your SurrealDB instance                     |
| `LN_DB_DATABASE`          | SurrealDB database                                     |
| `LN_DB_NAMESPACE`         | SurrealDB namespace                                    |
| `LN_DB_USERNAME`          | SurrealDB username                                     |
| `LN_DB_PASSWORD`          | SurrealDB password                                     |
