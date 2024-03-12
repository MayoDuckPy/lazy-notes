use cfg_if::cfg_if;

// Tell rustc that components use ssr with islands enabled
cfg_if! { if #[cfg(feature = "ssr")] {
use axum_session_auth::{AuthSession, SessionSurrealPool};
use ammonia::{is_html, Builder};
use crate::auth;
use crate::settings::LazyNotesSettings;
use html5ever::{
    ATOM_LOCALNAME__68_31 as TOKEN_H1,
    ATOM_LOCALNAME__68_32 as TOKEN_H2,
    ATOM_LOCALNAME__68_33 as TOKEN_H3,
    ATOM_LOCALNAME__68_34 as TOKEN_H4,
    ATOM_LOCALNAME__68_35 as TOKEN_H5,
    ATOM_LOCALNAME__68_36 as TOKEN_H6,
    ATOM_LOCALNAME__63_6C_61_73_73 as TOKEN_CLASS,
    ATOM_LOCALNAME__69_64 as TOKEN_ID,
    tendril::StrTendril,
    tokenizer::{BufferQueue, Token, Tokenizer, TokenSink, TokenSinkResult, Tag, TagKind},
};
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

#[derive(Clone, Debug, PartialEq)]
pub struct TocHeading {
    level: u8,
    class: Option<Box<str>>,
    id: Option<Box<str>>,
    text: Option<Box<str>>
}

impl TocHeading {
    fn set_text(&mut self, text: &str) {
        self.text = Some(Box::from(text));
    }
}

struct TocSink {
    headings: Vec<TocHeading>
}

impl TokenSink for TocSink {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            Token::TagToken(Tag {
                kind: TagKind::StartTag,
                name,
                self_closing: false,
                attrs
            }) => {
                if [TOKEN_H1, TOKEN_H2, TOKEN_H3, TOKEN_H4, TOKEN_H5, TOKEN_H6].contains(&name) {
                    let level = match name {
                        TOKEN_H1 => 1,
                        TOKEN_H2 => 2,
                        TOKEN_H3 => 3,
                        TOKEN_H4 => 4,
                        TOKEN_H5 => 5,
                        TOKEN_H6 => 6,
                        _ => 0,  // Impossible
                    };

                    let class = attrs
                        .iter()
                        .find(|a| a.name.local == TOKEN_CLASS)
                        .map(|a| Some(Box::from(a.value.to_string())))
                        .unwrap_or_else(|| None);

                    let id = attrs
                        .iter()
                        .find(|a| a.name.local == TOKEN_ID)
                        .map(|a| Some(Box::from(a.value.to_string())))
                        .unwrap_or_else(|| None);

                    self.headings.push(TocHeading {
                        level,
                        class,
                        id,
                        text: None
                    });
                }
            },
            Token::CharacterTokens(string) => {
                if let Some(heading) = self.headings.last_mut() {
                    if heading.text.is_some() {
                        return TokenSinkResult::Continue;
                    }

                    heading.set_text(&string);
                }
            }
            _ => {},
        };

        TokenSinkResult::Continue
    }
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
pub fn TocSidebar(toc: Vec<TocHeading>) -> impl IntoView {
    if toc.is_empty() {
        return view! {
            <nav id="toc_wrapper">
                <ul id="toc"/>
            </nav>
        }
    }

    let mut toc_tree = String::new();
    let mut last_heading: u8 = toc[0].level;
    let mut nest_count: u8 = 0;

    // Construct TOC list manually (safe because classes and ids are sanitized)
    for heading in toc {
        let id = heading.id.unwrap_or_else(|| "".into());
        let text = heading.text.clone().unwrap_or_else(|| "".into());

        if heading.level > last_heading {
            toc_tree.push_str(&format!("<li><ul><li><a href=#{id}>{text}</a></li>"));
            nest_count += 1;
        } else if heading.level < last_heading && nest_count > 0 {
            toc_tree.push_str(&format!("</ul></li><li><a href=#{id}>{text}</a></li>"));
            nest_count -= 1;
        } else {
            toc_tree.push_str(&format!("<li><a href=#{id}>{text}</a></li>"));
        }

        last_heading = heading.level;
    }

    // Close any unclosed nested lists
    while nest_count > 0 {
        toc_tree.push_str("</ul></li>");
        nest_count -= 1;
    }

    view! {
        <nav id="toc_wrapper">
            <ul id="toc" inner_html=toc_tree/>
        </nav>
    }
}

#[component]
pub fn Navbar() -> impl IntoView {
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    let toc_context: Option<Vec<TocHeading>> = use_context();
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
    let response = send_signup.value();

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

                <ErrorBoundary
                    fallback=move |_| view! {
                        <p class="error">"Incorrect field(s)"</p>
                    }>
                    <p>{response}</p>
                </ErrorBoundary>

                <input type="submit" value="Submit"/>
            </ActionForm>
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

                <ErrorBoundary
                    fallback=move |_| view! {
                        <p class="error">"Incorrect username or password"</p>
                    }>
                    <p>{response}</p>
                </ErrorBoundary>

                <input type="submit" value="Submit"/>
            </ActionForm>
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
        let mut notes = String::new();
        if let Ok(path) = params.with(|params| params.clone().map(move |params| params.path.clone())) {
            let mut ext = String::new();
            if &path[path.len() - 3..] != ".md" {
                ext = "/index.md".to_string();
            }

            notes = match read_to_string(format!("{}/{}/notes/{path}{ext}", &ln_settings.data_dir, &user.username)) {
                Ok(notes) => notes,
                Err(_) => format!("Error reading file!").to_string(),
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

/// Generate a table of contents from HTML by parsing heading elements.
fn generate_toc(html: &str) -> Result<Vec<TocHeading>, String> {
    if !is_html(html) {
        return Err("Invalid HTML".to_string());
    }

    let sink = TocSink { headings: Vec::new() };

    // Prepare input
    let mut input = BufferQueue::new();
    input.push_back(StrTendril::from_slice(html));

    // Parse
    let mut tokenizer = Tokenizer::new(sink, Default::default());
    let _ = tokenizer.feed(&mut input);
    tokenizer.end();

    Ok(tokenizer.sink.headings)
}
}}

#[cfg(feature = "ssr")]
#[cfg(test)]
mod tests {
    use crate::app::{generate_toc, TocHeading};

    #[test]
    fn test_toc_generation() {
        let invalid_html = "# Markdown Title";
        let basic_html = r#"<h1 id="test"></h1>"#;
        let html_no_id = r#"<h1></h1>"#;
        let html_only_class = r#"<h1 class="test"></h1>"#;
        let html_with_class_at_start = r#"<h1 class="class" id="test"></h1>"#;
        let html_with_class_at_end = r#"<h1 id="test" class="class"></h1>"#;
        let html_with_text = r#"<h1>test</h1>"#;
        let html_with_text_and_attrs = r#"<h1 id="test" class="test">test</h1>"#;
        let long_html = r#"<h1 id="test"></h1><h2 class="hello">hello</h2><h5 id="test2" class="success"></h5><h6 class="success" id="test3">test3</h6>"#;

        assert_eq!(generate_toc(invalid_html), Err("Invalid HTML".to_string()));

        assert_eq!(
            generate_toc(basic_html),
            Ok(vec![TocHeading {
                level: 1,
                class: None,
                id: Some(Box::from("test")),
                text: None
            }])
        );

        assert_eq!(
            generate_toc(html_no_id),
            Ok(vec![TocHeading {
                level: 1,
                class: None,
                id: None,
                text: None
            }])
        );

        assert_eq!(
            generate_toc(html_only_class),
            Ok(vec![TocHeading {
                level: 1,
                class: Some(Box::from("test")),
                id: None,
                text: None
            }])
        );

        assert_eq!(
            generate_toc(html_with_class_at_start),
            Ok(vec![TocHeading {
                level: 1,
                class: Some(Box::from("class")),
                id: Some(Box::from("test")),
                text: None
            }])
        );

        assert_eq!(
            generate_toc(html_with_class_at_end),
            Ok(vec![TocHeading {
                level: 1,
                class: Some(Box::from("class")),
                id: Some(Box::from("test")),
                text: None
            }])
        );

        assert_eq!(
            generate_toc(html_with_text),
            Ok(vec![TocHeading {
                level: 1,
                class: None,
                id: None,
                text: Some(Box::from("test"))
            }])
        );

        assert_eq!(
            generate_toc(html_with_text_and_attrs),
            Ok(vec![TocHeading {
                level: 1,
                class: Some(Box::from("test")),
                id: Some(Box::from("test")),
                text: Some(Box::from("test"))
            }])
        );

        assert_eq!(
            generate_toc(long_html),
            Ok(vec![
                TocHeading {
                    level: 1,
                    class: None,
                    id: Some(Box::from("test")),
                    text: None
                },
                TocHeading {
                    level: 2,
                    class: Some(Box::from("hello")),
                    id: None,
                    text: Some(Box::from("hello"))
                },
                TocHeading {
                    level: 5,
                    class: Some(Box::from("success")),
                    id: Some(Box::from("test2")),
                    text: None
                },
                TocHeading {
                    level: 6,
                    class: Some(Box::from("success")),
                    id: Some(Box::from("test3")),
                    text: Some(Box::from("test3"))
                }
            ])
        );
    }
}
