use ammonia::Builder;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pulldown_cmark::{html, Options, Parser};
use std::fs::read_to_string;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/lazy-notes.css"/>

        // sets the document title
        <Title text="Lazy Notes"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/user/notes/*path" view=Note/>
                </Routes>
            </main>
        </Router>
    }
}

// TODO: Create db tables
// TODO: Setup account auth
// TODO: Setup cache
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <section>
            <h1>"Lazy Notes"</h1>
        </section>
    }
}

#[component]
pub fn Note() -> impl IntoView {
    let params = use_params_map();
    let notes_as_html = move || {
        let mut notes = "File not found".to_string();
        if let Some(path) = params.with(|params| params.get("path").cloned()) {
            let mut ext = String::new();
            if &path[path.len() - 3..] != ".md" {
                ext = "/index.md".to_string();
            }

            notes = match read_to_string(format!("/path/to/notes/dir/{path}{ext}")) {
                Ok(notes) => notes,
                Err(e) => format!("Error reading file: {e}").to_string(),
            };

            // Process urls to reflect current user
            notes = notes.replace("](/resources", "](/user/resources");
            notes = notes.replace("src=\"/resources", "src=\"/user/resources");
        }

        convert_to_html(&notes)
    };

    view! {
        <section>
            <nav>"Lazy Notes"</nav>
            <article id="notes" inner_html=notes_as_html/>
        </section>
    }
}

fn convert_to_html(md_input: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(md_input, options);

    let mut dirty_md = String::new();
    html::push_html(&mut dirty_md, parser);

    // TODO: Allow specifying allowed tags in settings.toml
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
