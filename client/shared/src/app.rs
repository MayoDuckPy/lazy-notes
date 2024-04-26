use crux_core::{macros::Effect, render::Render, App};
use crux_http::Http;
use crux_kv::{KeyValue, KeyValueOutput};
use serde::{Deserialize, Serialize};

use crate::auth::{handle_login, login, Session};
use crate::note::{display_note, get_note};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    // Notes
    Clear,
    GetNote,
    #[serde(skip)]
    DisplayNote(crux_http::Result<crux_http::Response<String>>),

    // Authentication
    GetSession,
    Login(Box<str>, Box<str>, Box<str>),
    #[serde(skip)]
    HandleLogin(crux_http::Result<crux_http::Response<Vec<u8>>>, Box<str>),

    // Session management
    // RestoreState,
    #[serde(skip)]
    LoadSession(KeyValueOutput),
    #[serde(skip)]
    SaveSession(KeyValueOutput),
}

#[derive(Clone, Default)]
pub struct Model {
    pub note: Option<Box<str>>,
    pub session: Option<Session>,
    // pub instance: Option<Settings>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub note: Box<str>,
    pub is_logged_in: bool,
    // pub instance: Box<str>,  // TODO: Show in settings view
}

#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(Effect)]
#[effect(app = "Note")]
pub struct Capabilities {
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
            Event::GetNote => get_note(model, caps),
            Event::DisplayNote(response) => display_note(model, caps, response),

            Event::GetSession => caps.key_value.read("session", Event::LoadSession),
            Event::Login(instance, username, password) => login(caps, instance, username, password),
            Event::HandleLogin(response, instance) => handle_login(model, caps, response, instance),

            Event::LoadSession(KeyValueOutput::Read(Some(session))) => {
                model.session = serde_json::from_slice(&session).ok();

                // Update view model
                caps.render.render();
            }
            Event::LoadSession(KeyValueOutput::Read(None)) => {}
            Event::LoadSession(KeyValueOutput::Write(_success)) => unreachable!(),
            Event::SaveSession(_) => {} // Assume correctness (Android should not trigger error case)
        };
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            note: model
                .note
                .clone()
                .unwrap_or_else(|| "No note available".into()),
            is_logged_in: model.session.is_some(),
        }
    }
}
