use cfg_if::cfg_if;

// Tell rustc that components use ssr with islands enabled
cfg_if! { if #[cfg(feature = "ssr")] {
use axum_session_auth::{AuthSession, SessionSurrealPool};
use ammonia::Builder;
use crate::auth;
use crate::settings::LazyNotesSettings;
use http::StatusCode;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
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

    view! {
        <Stylesheet id="leptos" href="/pkg/lazy-notes.css"/>
        <Title text="Lazy Notes"/>

        <Router>
            <main>
                <Routes>
                    // NOTE: This component is not receiving global context
                    //       defined in 'main.rs' so redirect to /home is
                    //       needed to obtain AuthSession.
                    <Route path="" view=|| leptos_axum::redirect("/home")/>
                    <Route path="/home" view=HomePage/>
                    <Route path="/signup" view=Signup/>
                    <Route path="/login" view=Login/>
                    <Route path="/:user/notes" view=|| view! { <Outlet/> }>
                        <Route path="" view=|| leptos_axum::redirect("notes/index.md")/>
                        <Route path="*path" view=Note/>
                    </Route>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Navbar() -> impl IntoView {
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    let send_logout = create_server_action::<auth::Logout>();

    let logo_link = {
        if auth.is_authenticated() {
            format!("/{}/notes/index.md", &auth.current_user.clone().expect("User not authenticated").username)
        } else {
            "/home".to_string()
        }
    };

    // TODO: Hide nav on scroll down
    // TODO: Add hamburger menu to open navigational sidebar on Notes component
    view! {
        <nav class="header_nav">
            <section class="left_nav">
                <A href={logo_link}>"Lazy Notes"</A>
            </section>
            // <section class="middle_nav">
            // </section>
            <section class="right_nav">
                {move || if auth.is_authenticated() {
                    view! {
                        <ActionForm action=send_logout>
                            <input class="logout_btn" type="submit" value="Log out"/>
                        </ActionForm>
                    }.into_view()
                } else {
                    view! {
                        <A class="login_btn" href="/login">"Log in"</A>
                        <A class="signup_btn" href="/signup">"Sign up"</A>
                    }.into_view()
                }}
            </section>
        </nav>
    }
}

#[component]
pub fn Signup() -> impl IntoView {
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    // If authenticated, redirect to user notes page
    if auth.is_authenticated() {
        auth.current_user.and_then(|user| {
            leptos_axum::redirect(&format!("/{}/notes/index.md", &user.username));
            Some(())
        });
    }

    let send_signup = create_server_action::<auth::Signup>();

    view! {
        <article class="signup">
            <ActionForm action=send_signup>
                <label>
                    "Username"
                    <input name="username" pattern="[a-zA-Z0-9_-]*"/>
                </label>
                <label>
                    "Password"
                    <input name="password" type="password"/>
                </label>
                <label>
                    "Password Confirmation"
                    <input name="password_confirmation" type="password"/>
                </label>
                <input type="submit" value="Submit"/>
            </ActionForm>
            <ErrorBoundary fallback=move |_| view! { <p>"Incorrect field(s)"</p>}>
                <p></p>
            </ErrorBoundary>
        </article>
    }
}

#[component]
pub fn Login() -> impl IntoView {
    // let response: ResponseOptions = expect_context();
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    // If authenticated, redirect to user notes page
    if auth.is_authenticated() {
        auth.current_user.and_then(|user| {
            leptos_axum::redirect(&format!("/{}/notes/index.md", &user.username));
            Some(())
        });
    }

    let send_login = create_server_action::<auth::Login>();
    let response = send_login.value();

    view! {
        <article class="login">
            <ActionForm action=send_login>
                <label>
                    "Username"
                    <input name="username" pattern="[a-zA-Z0-9_-]*"/>
                </label>
                <label>
                    "Password"
                    <input name="password" type="password"/>
                </label>
                // <label>
                //     "Remember Me"
                //     <input name="remember" type="radio"/>
                // </label>
                <input type="submit" value="Submit"/>
            </ActionForm>
            <ErrorBoundary fallback=move |_| view! { <p>"Incorrect login"</p>}>
                <p>{response}</p>
            </ErrorBoundary>
        </article>
    }
}

// TODO: Setup cache
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Navbar/>
        <article class="welcome">
            <h1>"Welcome to Lazy Notes"</h1>
        </article>
    }
}

#[component]
pub fn Note() -> impl IntoView {
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();
    let response: ResponseOptions = expect_context();

    if !auth.is_authenticated() {
        response.set_status(StatusCode::UNAUTHORIZED);
        return view! { <Unauthorized/> };
    }

    let user = auth.current_user.clone().expect("User is authenticated");
    let ln_settings = use_context::<LazyNotesSettings>().expect("Failed to get configuration context");

    // TODO: Is it possible to not clone 'params'?
    let params = use_params::<NotesParams>();
    if let Ok(username) = params.with(|params| params.clone().map(move |params| params.user.clone())) {
        if username != user.username {
            response.set_status(StatusCode::UNAUTHORIZED);
            return view! { <Unauthorized/> };
        }
    }

    let notes_as_html = move || {
        let mut notes = "File not found".to_string();
        if let Ok(path) = params.with(|params| params.clone().map(move |params| params.path.clone())) {
            let mut ext = String::new();
            if &path[path.len() - 3..] != ".md" {
                ext = "/index.md".to_string();
            }

            notes = match read_to_string(format!("{}/{}/notes/{path}{ext}", &ln_settings.data_dir, &user.username)) {
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
        <Navbar/>
        <article id="notes_wrapper">
            <article id="notes" inner_html=notes_as_html/>
        </article>
    }.into_view()
}

#[component]
pub fn Unauthorized() -> impl IntoView {
    view! {
        <article class="no_permission">
            <p>"You do not have permission to view this page."</p>
        </article>
    }
}

/// Handles sanitizing and converting markdown to html.
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
