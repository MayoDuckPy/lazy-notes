# Lazy Notes

Lazy notes is a note-taking app where markdown notes are lazily-rendered to
HTML (and optionally cached) when viewed. This project aims to be a modern,
lightweight and, FOSS-alternative to other note-taking apps.

## Roadmap

- Finish polishing the UI
- Setup auth
- Setup caching
- Build cross-platform client app
- Enable custom CSS files
- Setup multi-user editing
- Add more note-taking tools?
	- Kanban
	- Diagrams

## Building

> NOTE: Do not run this project in a production environment as debug flags are
currently still enabled. They can be disabled in the `Cargo.toml` file.

`cargo-leptos` is used to build the project:

```sh
cargo install --locked cargo-leptos
```

See the [cargo-leptos](https://github.com/leptos-rs/cargo-leptos/) repository
for more details.

<br>

Run the project with:

```sh
cargo leptos --release serve
```
