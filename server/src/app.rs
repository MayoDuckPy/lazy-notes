use cfg_if::cfg_if;

// Tell rustc that components use ssr with islands enabled
cfg_if! { if #[cfg(feature = "ssr")] {
use axum_session_auth::{AuthSession, SessionSurrealPool};
use ammonia::{is_html, Builder};
use crate::auth;
use crate::settings::LazyNotesSettings;
use http::StatusCode;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_axum::ResponseOptions;
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::{fs::read_to_string, sync::OnceLock};
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
pub fn Logo() -> impl IntoView {
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    // Construct logo
    let logo_link = {
        if auth.is_authenticated() {
            format!("/{}/notes/index.md", &auth.current_user.clone().expect("User not authenticated").username)
        } else {
            "/home".to_string()
        }
    };

    view! { <A id="logo" href={logo_link}>"Lazy Notes"</A> }
}

#[component]
pub fn TocSidebar(toc: Vec<(u8, Box<str>)>) -> impl IntoView {

    view! {
        <nav id="toc_wrapper">
            <ul id="toc">
             {toc.clone().into_iter()
                 .map(move |(heading, section)| view! { <li><a href={format!("#{section}")}>{format!("{heading}. {section}")}</a></li> })
                 .collect_view()}
            </ul>
        </nav>
    }
}

#[component]
pub fn Navbar() -> impl IntoView {
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    let toc_context: Option<Vec<(u8, Box<str>)>> = use_context();
    let toc_visible = toc_context.clone().is_some_and(|v| !v.is_empty());

    let send_logout = create_server_action::<auth::Logout>();

    view! {
        // Use JS as it is far easier than wrangling wasm_bindgen
        <Script>
        "
            let lastY = 0;
            let nav = null;

            window.onload = () => {
                lastY = window.scrollY;
                nav = document.querySelector('nav.header_nav');
            };

            document.onscroll = ev => {
                let isHidden = nav.classList[1] == 'hidden';
                let y = window.scrollY;
                if (y > lastY && !isHidden && !document.querySelector('#toc_state:checked')) {
                    nav.className = 'header_nav hidden';
                } else if (y < lastY && isHidden) {
                    nav.className = 'header_nav';
                }
                lastY = y;
            };
        "
        </Script>
        <Show when=move || toc_visible>
            // Having this outside instead of nested makes CSS easier
            <input type="checkbox" id="toc_state" style="display: none !important"/>
        </Show>
        <nav class="header_nav">
            <section class="left_nav">
                <Show when=move || toc_visible
                      fallback=Logo>
                    <label for="toc_state" id="toc_revealer">"≡"</label>
                </Show>
            </section>
            <section class="middle_nav">
                <Show when=move || toc_visible>
                    <Logo/>
                </Show>
            </section>
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
        <Show when=move || toc_visible>
            <TocSidebar toc=toc_context.clone().unwrap()/>
        </Show>
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
        <Navbar/>
        <article class="signup">
            <ActionForm action=send_signup>
                <h1>"Sign Up"</h1>
                <label for="username">"Username"</label>
                <input name="username" pattern="[a-zA-Z0-9_\\-]+" required/>

                <label for="password">"Password"</label>
                <input name="password" type="password" required/>

                <label for="password_confirmation">"Password Confirmation"</label>
                <input name="password_confirmation" type="password" required/>

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
        <Navbar/>
        <article class="login">
            <ActionForm action=send_login>
                <h1>"Log In"</h1>
                <label for="username">"Username"</label>
                <input name="username" pattern="[a-zA-Z0-9_\\-]+" required/>

                <label for="password">"Password"</label>
                <input name="password" type="password" required/>

                // <label for="remember">"Remember Me"</label>
                // <input name="remember" type="radio"/>

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

    let notes_as_html = {
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

    // Should only fail if bad HTML which should not happen due to sanitization
    let toc = generate_toc(&notes_as_html).unwrap();

    view! {
        <Provider value=toc>
            <Navbar/>
        </Provider>
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
        // TODO: Prefix classes and ids to prevent conflicts (strip prefix in TOC)
        .add_tag_attributes("h1", &["class", "id"])
        .add_tag_attributes("h2", &["class", "id"])
        .add_tag_attributes("h3", &["class", "id"])
        .add_tag_attributes("h4", &["class", "id"])
        .add_tag_attributes("h5", &["class", "id"])
        .add_tag_attributes("h6", &["class", "id"])
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

/// Get the table of contents from HTML by parsing heading element ids.
fn generate_toc(html: &str) -> Result<Vec<(u8, Box<str>)>, String> {
    /* NOTE: Uses regex instead of HTML parser as headings only have ids and classes.
             Use parser if more attributes are added. */
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| Regex::new(r#"<h([1-6])(?: class="[^"]*")? id="([^"]+)"(?: class="[^"]*")?>"#).expect("Invalid regex"));

    if !is_html(html) {
        return Err("Invalid HTML".to_string());
    }

    let mut toc = Vec::new();
    for (_, [heading, id]) in re.captures_iter(html).map(|c| c.extract()) {
        toc.push((heading.parse::<u8>().expect("Impossible HTML heading"), Box::from(id)));
    }

    Ok(toc)
}
}}

#[cfg(feature = "ssr")]
#[cfg(test)]
mod tests {
    use crate::app::generate_toc;

    #[test]
    fn test_toc_generation() {
        let invalid_html = "# Markdown Title";
        let basic_html = r#"<h1 id="test"></h1>"#;
        let html_no_id = r#"<h1></h1>"#;
        let html_only_class = r#"<h1 class="test"></h1>"#;
        let html_with_class_at_start = r#"<h1 class="class" id="test"></h1>"#;
        let html_with_class_at_end = r#"<h1 id="test" class="class"></h1>"#;
        let long_html = r#"<h1 id="test"></h1><h2 class="fail"></h2><h5 id="test2" class="success"></h5><h6 class="success" id="test3"></h6>"#;

        assert_eq!(generate_toc(invalid_html), Err("Invalid HTML".to_string()));
        assert_eq!(generate_toc(basic_html), Ok(vec![(1, Box::from("test"))]));
        assert_eq!(generate_toc(html_no_id), Ok(vec![]));
        assert_eq!(generate_toc(html_only_class), Ok(vec![]));
        assert_eq!(
            generate_toc(html_with_class_at_start),
            Ok(vec![(1, Box::from("test"))])
        );
        assert_eq!(
            generate_toc(html_with_class_at_end),
            Ok(vec![(1, Box::from("test"))])
        );
        assert_eq!(
            generate_toc(long_html),
            Ok(vec![
                (1, Box::from("test")),
                (5, Box::from("test2")),
                (6, Box::from("test3"))
            ])
        );
    }
}
