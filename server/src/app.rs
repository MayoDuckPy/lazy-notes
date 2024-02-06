use cfg_if::cfg_if;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

// Tell rustc that components use ssr with islands enabled
cfg_if! { if #[cfg(feature = "ssr")] {
use axum_session_auth::{AuthSession, SessionSurrealPool};
use ammonia::Builder;
use crate::auth;
use crate::settings;
use crate::settings::LazyNotesSettings;
use http::StatusCode;
use leptos_axum::ResponseOptions;
use pulldown_cmark::{html, Options, Parser};
use std::fs::read_to_string;
use surrealdb::{engine::remote::ws::Client, Surreal};

#[derive(Clone, Params, PartialEq)]
struct NotesParams {
    user: String,
    path: String,
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Get Lazy Notes configuration
    let ln_settings = settings::get_configuration(None).expect("Failed to read configuration file");
    provide_context(ln_settings);

    view! {
        <Stylesheet id="leptos" href="/pkg/lazy-notes.css"/>
        <Title text="Lazy Notes"/>

        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/test" view=Test/>
                    <Route path="/:user/notes/*path" view=Note/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Test() -> impl IntoView {
    let response: ResponseOptions = expect_context();
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    if !auth.is_authenticated() {
        response.set_status(StatusCode::UNAUTHORIZED);
        return view! {
            <p>"Please login to view this page"</p>
        }
    }

    let user = auth.current_user.clone().unwrap_or_default();
    view! {
        <p>"Current user: "{move || user.username.clone()}</p>
    }
}

// TODO: Setup cache
#[component]
pub fn HomePage() -> impl IntoView {
    let send_login = create_server_action::<auth::Login>();
    let send_logout = create_server_action::<auth::Logout>();
    let response = send_login.value();
    view! {
        <article>
            <h1>"Lazy Notes"</h1>
            <A href="/test">"Test page"</A>
            <br/>
            <ActionForm action=send_login>
                <label>
                    "Username"
                    // TODO: Prevent invalid username inputs [a-zA-Z0-9_]*
                    <input name="username"/>
                </label>
                <label>
                    "Password"
                    <input name="password" type="password"/>
                </label>
                <input type="submit" value="Submit"/>
            </ActionForm>
            <ErrorBoundary fallback=move |_| view! { <p>"Incorrect login"</p>}>
                <p>{response}</p>
            </ErrorBoundary>

            <br/>
            // TODO: Move to nav and make appear after login
            <ActionForm action=send_logout>
                <input type="submit" value="Logout"/>
            </ActionForm>
        </article>
    }
}

#[component]
pub fn Note() -> impl IntoView {
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();
    let response: ResponseOptions = expect_context();

    let err_view = view! {
        <article class="no_permission">
            <p>"You do not have permission to view this page"</p>
        </article>
    };

    if !auth.is_authenticated() {
        response.set_status(StatusCode::UNAUTHORIZED);
        return err_view;
    }

    let user = auth.current_user.clone().expect("User is authenticated");
    let ln_settings = use_context::<LazyNotesSettings>().expect("Failed to get configuration context");

    // TODO: Is it possible to not clone 'params'?
    let params = use_params::<NotesParams>();
    if let Ok(username) = params.with(|params| params.clone().map(move |params| params.user.clone())) {
        if username != user.username {
            response.set_status(StatusCode::UNAUTHORIZED);
            return err_view;
        }
    }

    let notes_as_html = move || {
        let mut notes = "File not found".to_string();
        if let Ok(path) = params.with(|params| params.clone().map(move |params| params.path.clone())) {
            let mut ext = String::new();
            if &path[path.len() - 3..] != ".md" {
                ext = "/index.md".to_string();
            }

            notes = match read_to_string(format!("{}/{}/notes/{path}{ext}", &ln_settings.notes_dir, &user.username)) {
                Ok(notes) => notes,
                Err(e) => format!("Error reading file: {e}").to_string(),
            };

            // Process urls to reflect current user
            notes = notes.replace("](/resources", &format!("](/{}/resources", &user.username));
            notes = notes.replace("src=\"/resources", &format!("src=\"/{}/resources", &user.username));
        }

        convert_to_html(&notes)
    };

    view! {
        <article>
            <nav>"Lazy Notes"</nav>
            <article id="notes" inner_html=notes_as_html/>
        </article>
    }
}

fn convert_to_html(md_input: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(md_input, options);

    let mut dirty_md = String::new();
    html::push_html(&mut dirty_md, parser);

    // TODO: Allow specifying allowed tags in settings.toml
    // TODO: Add MathML specs
    Builder::default()
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
