use cfg_if::cfg_if;

// Tell rustc that components use ssr with islands enabled
cfg_if! { if #[cfg(feature = "ssr")] {
use axum_session_auth::{AuthSession, SessionSurrealPool};
use ammonia::is_html;
use crate::api::get_note_as_html;
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
                        <Route
                            path="*path"
                            view=Note
                            ssr=SsrMode::PartiallyBlocked
                        />
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
pub fn Navbar(
    #[prop(default = None)]
    toc: Option<Vec<TocHeading>>
) -> impl IntoView {
    let ln_settings: LazyNotesSettings = expect_context();
    let auth: AuthSession<auth::User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        expect_context();

    let toc_visible = toc.as_ref().is_some_and(|v| !v.is_empty());
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
                        {move || ln_settings.enable_registration.then(||
                            view! { <A class="signup_btn" href="/signup">"Sign up"</A> })
                        }
                    }.into_view()
                }}
            </section>
        </nav>
        <Show when=move || toc_visible>
            // TODO: Proper error handling
            // Should only fail if bad HTML which should not happen due to sanitization
            <TocSidebar toc=toc.clone().expect("Invalid HTML while parsing headings")/>
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
                    fallback=move |errors| {
                        errors.get()
                            .into_iter()
                            .map(|(_, e)| view! {
                                <p class="error">
                                {format!("{}", e.to_string()
                                    .strip_prefix("error running server function: ")
                                    .unwrap_or_else(|| "Incorrect field(s)"))}
                                </p>
                            }).collect_view()
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

                <fieldset>
                    <input name="remember" type="checkbox"/>
                    <label for="remember">"Remember Me"</label>
                </fieldset>

                <ErrorBoundary
                    fallback=move |errors| {
                        errors.get()
                            .into_iter()
                            .map(|(_, e)| view! {
                                <p class="error">
                                {format!("{}", e.to_string()
                                    .strip_prefix("error running server function: ")
                                    .unwrap_or_else(|| "Incorrect username or password"))}
                                </p>
                            }).collect_view()
                    }>
                    <p>{response}</p>
                </ErrorBoundary>

                <input type="submit" value="Submit"/>
            </ActionForm>
        </article>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Navbar/>
        <article class="welcome">
            <h1>"Welcome to Lazy Notes"</h1>
            <p>"Lazy Notes is a web frontend for your markdown notes."</p>
            <p>"Markdown files are lazily rendered to HTML when viewed allowing you to instantly see any changes you make to your files locally. Making use of Rust and the Leptos web framework, this project aims to be an extremely" <b>" fast "</b> "and" <b>" lightweight "</b> "option for securely viewing your notes on the web."</p>
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

    let user = auth.current_user.clone().expect("User was not authenticated");
    let params = use_params::<NotesParams>();

    // Check if user may access current note
    if !params.with(|params|
        params.as_ref()
            .map(|params| user.username == params.user)
            .is_ok_and(|authenticated| authenticated))
    {
        response.set_status(StatusCode::UNAUTHORIZED);
        return view! { <Unauthorized/> };
    }

    let notes_as_html = create_blocking_resource(move || (), move |_| {
        let path = params.get().map(|params| params.path).unwrap_or("".into());
        async move { get_note_as_html(path).await }}
    );

    view! {
        <Suspense fallback=move || view! {
            <article id="notes_wrapper">
                <p>"Getting your notes..."</p>
            </article>
        }>
            <Navbar toc=notes_as_html.get()
                .and_then(|notes| notes.ok())
                .and_then(|notes| generate_toc(&notes).ok())/>
            <article id="notes_wrapper">
                {move || notes_as_html.get()
                    .transpose()
                    .map_err(|e| {
                        view! {
                            <article id="notes_error">
                                <p>
                                {move || e.to_string()
                                    .strip_prefix("error running server function: ")
                                    .unwrap_or_else(|| "Failed to get note")
                                    .to_owned()}
                                </p>
                            </article>
                        }
                    })
                    .map(|notes| view! { <article id="notes" inner_html=notes/> })
                    .unwrap_or_else(|e| e)
                }
            </article>
        </Suspense>
    }.into_view()
}

#[component]
pub fn Unauthorized() -> impl IntoView {
    view! {
        <Navbar/>
        <article class="no_permission">
            <p>"You do not have permission to view this page."</p>
        </article>
    }
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
    fn toc_generation() {
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
