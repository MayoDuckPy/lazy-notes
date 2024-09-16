use cfg_if::cfg_if;
use leptos::*;

cfg_if! { if #[cfg(feature = "ssr")] {
use ammonia::Builder;
use axum_session_auth::AuthSession;
use axum_session_surreal::SessionSurrealPool;
use crate::auth;
use crate::settings::LazyNotesSettings;
use http::StatusCode;
use leptos_axum::ResponseOptions;
use pulldown_cmark::{html, Options, Parser};
use std::fs::read_to_string;
use surrealdb::{engine::remote::ws::Client, Surreal};

/// Handles sanitizing and converting markdown to html.
fn convert_to_html(md_input: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(md_input, options);

    let mut dirty_md = String::new();
    html::push_html(&mut dirty_md, parser);

    // TODO: Allow specifying allowed tags in settings.toml
    // TODO: Add MathML specs
    Builder::default()
        // .allowed_classes()
        .id_prefix(Some("ln-"))
        .add_tag_attributes("h1", &["id"])
        .add_tag_attributes("h2", &["id"])
        .add_tag_attributes("h3", &["id"])
        .add_tag_attributes("h4", &["id"])
        .add_tag_attributes("h5", &["id"])
        .add_tag_attributes("h6", &["id"])
        .add_tags(&["audio"])
        .add_tag_attributes("video", &["src", "autoplay", "loop", "controls", "muted"])
        .add_tags(&["video"])
        .add_tag_attributes(
            "video",
            &["src", "autoplay", "loop", "controls", "muted", "width"],
        )
        .clean(&dirty_md)
        .to_string()
}
}}

#[server(endpoint = "get_note_as_html")]
pub async fn get_note_as_html(path: String) -> Result<String, ServerFnError> {
    // TODO: Write tests
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();
    let response: ResponseOptions = expect_context();

    if !auth.is_authenticated() {
        response.set_status(StatusCode::UNAUTHORIZED);
        return Err(ServerFnError::new("Authentication required"));
    }

    let user = auth.current_user.expect("User was not authenticated");
    let ln_settings: LazyNotesSettings = expect_context();

    let ext = {
        match &path[path.len() - 3..] != ".md" {
            true => "/index.md".to_string(),
            false => String::new(),
        }
    };

    // Get notes and process urls to reflect current user
    let notes = read_to_string(format!(
        "{}/{}/notes/{path}{ext}",
        &ln_settings.data_dir, &user.username
    ))
    .map_err(|_| ServerFnError::new("Error reading markdown file"))?
    .replace("](/resources", &format!("](/{}/resources", &user.username))
    .replace(
        "src=\"/resources",
        &format!("src=\"/{}/resources", &user.username),
    );

    Ok(convert_to_html(&notes))
}
