use crux_core::{macros::Effect, render::Render, App};
use crux_http::Http;
use crux_kv::{KeyValue, KeyValueOutput};
use html5ever::{
    tendril::StrTendril,
    tokenizer::{BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer},
    ATOM_LOCALNAME__63_6C_61_73_73 as TOKEN_CLASS, ATOM_LOCALNAME__68_31 as TOKEN_H1,
    ATOM_LOCALNAME__68_32 as TOKEN_H2, ATOM_LOCALNAME__68_33 as TOKEN_H3,
    ATOM_LOCALNAME__68_34 as TOKEN_H4, ATOM_LOCALNAME__68_35 as TOKEN_H5,
    ATOM_LOCALNAME__68_36 as TOKEN_H6, ATOM_LOCALNAME__69_64 as TOKEN_ID,
};
use serde::{Deserialize, Serialize};

use crate::auth::{handle_login, login, Session};
use crate::note::{display_note, get_note, get_note_css, render_css};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    // Notes
    Clear,
    GetNote(Box<str>),
    #[serde(skip)]
    DisplayNote(crux_http::Result<crux_http::Response<String>>),

    GetCss,
    #[serde(skip)]
    RenderCss(crux_http::Result<crux_http::Response<String>>),

    // Authentication
    GetSession,
    Login(Box<str>, Box<str>, Box<str>),
    #[serde(skip)]
    HandleLogin(
        crux_http::Result<crux_http::Response<Vec<u8>>>,
        Box<str>,
        Box<str>,
    ),

    // Session management
    // RestoreState,
    #[serde(skip)]
    LoadSession(KeyValueOutput),
    #[serde(skip)]
    SaveSession(KeyValueOutput),
}

#[derive(Clone, Default)]
pub struct Model {
    pub css: Option<Box<str>>,
    pub note: Option<Box<str>>,
    pub session: Option<Session>,
    // pub instance: Option<Settings>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub css: Option<Box<str>>,
    pub toc: Option<Vec<TocHeading>>,
    pub note: Option<Box<str>>,
    pub session: Option<Session>,
    // pub instance: Box<str>,  // TODO: Show in settings view
}

#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(Effect)]
#[effect(app = "Note")]
pub struct Capabilities {
    // pub html_parser: HtmlParser<Event>,
    pub http: Http<Event>,
    pub key_value: KeyValue<Event>,
    pub render: Render<Event>,
}

#[derive(Default)]
pub struct Note;

impl App for Note {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::Clear => {
                model.note = None;
                caps.render.render();
            }
            Event::GetNote(path) => get_note(model, caps, &path),
            Event::DisplayNote(response) => display_note(model, caps, response),

            Event::GetCss => get_note_css(model, caps),
            Event::RenderCss(response) => render_css(model, caps, response),

            Event::GetSession => caps.key_value.read("session", Event::LoadSession),
            Event::Login(instance, username, password) => login(caps, instance, username, password),
            Event::HandleLogin(response, instance, username) => {
                handle_login(model, caps, response, instance, username)
            }

            Event::LoadSession(KeyValueOutput::Read(Some(session))) => {
                model.session = serde_json::from_slice(&session).ok();

                // Update view model
                caps.render.render();
            }
            Event::LoadSession(KeyValueOutput::Read(None)) => {
                // No session available but update view model (session is None by default)
                caps.render.render();
            }
            Event::LoadSession(KeyValueOutput::Write(_success)) => unreachable!(),
            Event::SaveSession(_) => {} // Assume correctness (Android should not trigger error case)
        };
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            toc: model.note.as_ref().map(|note| generate_toc(note).ok()),
            css: model.css.clone(),
            note: model.note.clone(),
            session: model.session.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TocHeading {
    level: u8,
    class_name: Option<Box<str>>,
    id: Option<Box<str>>,
    text: Option<Box<str>>,
}

impl TocHeading {
    fn set_text(&mut self, text: &str) {
        self.text = Some(Box::from(text));
    }
}

struct TocSink {
    headings: Vec<TocHeading>,
}

impl TokenSink for TocSink {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            Token::TagToken(Tag {
                kind: TagKind::StartTag,
                name,
                self_closing: false,
                attrs,
            }) => {
                if [TOKEN_H1, TOKEN_H2, TOKEN_H3, TOKEN_H4, TOKEN_H5, TOKEN_H6].contains(&name) {
                    let level = match name {
                        TOKEN_H1 => 1,
                        TOKEN_H2 => 2,
                        TOKEN_H3 => 3,
                        TOKEN_H4 => 4,
                        TOKEN_H5 => 5,
                        TOKEN_H6 => 6,
                        _ => 0, // Impossible
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
                        class_name: class,
                        id,
                        text: None,
                    });
                }
            }
            Token::CharacterTokens(string) => {
                if let Some(heading) = self.headings.last_mut() {
                    if heading.text.is_some() {
                        return TokenSinkResult::Continue;
                    }

                    heading.set_text(&string);
                }
            }
            _ => {}
        };

        TokenSinkResult::Continue
    }
}

/// Generate a table of contents from HTML by parsing heading elements.
fn generate_toc(html: &str) -> Result<Vec<TocHeading>, String> {
    let sink = TocSink {
        headings: Vec::new(),
    };

    // Prepare input
    let mut input = BufferQueue::default();
    input.push_back(StrTendril::from_slice(html));

    // Parse
    let mut tokenizer = Tokenizer::new(sink, Default::default());
    let _ = tokenizer.feed(&mut input);
    tokenizer.end();

    Ok(tokenizer.sink.headings)
}
