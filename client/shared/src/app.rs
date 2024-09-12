use crux_core::{macros::Effect, render::Render, App};
use crux_http::Http;
use crux_kv::{KeyValue, KeyValueOutput};
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
            css: model.css.clone(),
            note: model.note.clone(),
            session: model.session.clone(),
        }
    }
}
