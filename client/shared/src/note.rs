use crux_http::{
    http::mime::{CSS, HTML},
    HttpError, Response,
};
use serde::{Deserialize, Serialize};

use crate::{Capabilities, Event, Model};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MediaBuffer {
    pub mime_type: Box<str>,
    pub buffer: Vec<u8>,
}

pub fn get_note(model: &mut Model, caps: &Capabilities, path: &str) {
    let session = match model.session.as_ref() {
        Some(session) => session,
        None => return,
    };

    caps.http
        .get(format!(
            "{}/{}/notes{path}",
            session.instance.as_ref(),
            session.username.as_ref()
        ))
        .content_type(HTML)
        .header("Cookie", format!("session={}", session.id.as_ref()))
        .expect_string()
        .send(Event::DisplayNote);
}

pub fn display_note(
    model: &mut Model,
    caps: &Capabilities,
    response: Result<Response<String>, HttpError>,
) {
    model.note = response
        .ok()
        .and_then(|mut res| res.take_body())
        .map(|body| body.into_boxed_str());

    // TODO: Notify user and return to login screen
    // Auth failed; send empty buffer to signal erasure of key
    if model.note.is_none() {
        caps.key_value.write("session", vec![], Event::SaveSession);
        model.session = None;
    }

    caps.render.render();
}

pub fn get_note_resource(model: &mut Model, caps: &Capabilities, path: &str) {
    let session = match model.session.as_ref() {
        Some(session) => session,
        None => return,
    };

    // TODO: Paths should implicitly incorporate /resources
    //       Should resources be put under the same directory as a note? (Probably)
    // No content-type request header; will get MIME from response
    caps.http
        .get(format!("{}{path}", session.instance.as_ref()))
        .header("Cookie", format!("session={}", session.id.as_ref()))
        .send(Event::RenderNoteResource);
}

// NOTE: Videos will be fully downloaded before viewing (no streaming)
pub fn render_note_resource(
    model: &mut Model,
    caps: &Capabilities,
    response: Result<Response<Vec<u8>>, HttpError>,
) {
    model.buffer = response
        .clone()
        .ok()
        .map(|mut res| {
            let media_buffer = MediaBuffer {
                mime_type: res.content_type()?.essence().into(),
                buffer: res.body_bytes().ok()?,
            };
            Some(media_buffer)
        })
        .flatten();

    caps.render.render();
}

pub fn get_note_css(model: &mut Model, caps: &Capabilities) {
    let session = match model.session.as_ref() {
        Some(session) => session,
        None => return,
    };

    caps.http
        .get(format!("{}/pkg/lazy-notes.css", session.instance.as_ref()))
        .content_type(CSS)
        .expect_string()
        .send(Event::RenderCss);
}

pub fn render_css(
    model: &mut Model,
    caps: &Capabilities,
    response: Result<Response<String>, HttpError>,
) {
    model.css = response
        .ok()
        .and_then(|mut res| res.take_body())
        .map(|body| body.into_boxed_str());

    caps.render.render();
}

#[cfg(test)]
mod note_tests {
    use core::panic;

    use crux_core::testing::AppTester;
    use crux_http::protocol::{HttpResponse, HttpResult};
    use crux_kv::KeyValueOutput;
    use html5ever::tendril::fmt::Slice;

    use crate::auth::Session;
    use crate::{Effect, Event, Model, Note};

    #[test]
    fn get_note() {
        let instance = "http://localhost:3000";
        let session_id = "sessionid123";
        let username = "login_test123";

        let app: AppTester<Note, _> = AppTester::default();
        let mut model = Model {
            css: None,
            buffer: None,
            note: None,
            session: None,
        };

        // Provide test session
        let _ = app.update(
            Event::LoadSession(KeyValueOutput::Read(Some(
                serde_json::to_vec(&Session {
                    id: session_id.into(),
                    instance: instance.into(),
                    username: username.into(),
                })
                .unwrap(),
            ))),
            &mut model,
        );

        assert!(model
            .session
            .as_ref()
            .is_some_and(|session| *session.id == *session_id));

        let mut update = app.update(Event::GetNote("/index.md".into()), &mut model);
        let req = match update.effects_mut().next().unwrap() {
            Effect::Http(req) => req,
            _ => panic!("Unexpected effect from event"),
        };

        // Cannot guarantee order of headers in test
        // assert_eq!(
        //     req.operation,
        //     HttpRequest::get(format!("{model.instance}/login_test/notes/index.md"))
        //         .header("content-type", "text/html;charset=utf-8")
        //         .header("cookie", "session=sessionid123")
        //         .build()
        // );

        let body = "<h1>Success</h1>";
        let res = app
            .resolve(req, HttpResult::Ok(HttpResponse::ok().body(body).build()))
            .unwrap();

        let display_event = res.events.get(0).unwrap().clone();
        let _ = app.update(display_event, &mut model);

        let view = app.view(&model);
        assert_eq!(view.note, Some(body.into()));
    }

    #[test]
    fn get_note_resource() {
        let instance = "http://localhost:3000";
        let session_id = "sessionid123";
        let username = "login_test123";

        let app: AppTester<Note, _> = AppTester::default();
        let mut model = Model {
            css: None,
            buffer: None,
            note: None,
            session: Some(Session {
                // Assume authenticated
                id: session_id.into(),
                instance: instance.into(),
                username: username.into(),
            }),
        };

        // Create resource fetch event
        let mut update = app.update(Event::GetNoteResource("/test.jpg".into()), &mut model);
        let req = match update.effects_mut().next().unwrap() {
            Effect::Http(req) => req,
            _ => panic!("Unexpected effect from event"),
        };

        // Resolve HTTP request
        let res = app
            .resolve(
                req,
                HttpResult::Ok(
                    HttpResponse::ok()
                        .header("content-type", "image/jpeg")
                        .body("test")
                        .build(),
                ),
            )
            .unwrap();

        // Handle returned resource
        let resource_load_event = res.events.get(0).unwrap().clone();
        let _ = app.update(resource_load_event, &mut model);

        match app.view(&model).buffer.as_ref() {
            Some(buffer) => {
                assert!(buffer.mime_type == "image/jpeg".into());
                assert!(buffer.buffer.as_bytes() == "test".as_bytes());
            }
            None => panic!("Buffer is None"),
        }
    }
}
