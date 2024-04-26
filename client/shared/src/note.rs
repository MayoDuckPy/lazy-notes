use crux_http::{http::mime::HTML, HttpError, Response};

use crate::{Capabilities, Event, Model};

const INDEX_SITE: &str = "/login_test/notes/index.md";

pub fn get_note(model: &mut Model, caps: &Capabilities) {
    let session = match model.session.as_ref() {
        Some(session) => session,
        None => return,
    };

    caps.http
        .get(format!("{}{INDEX_SITE}", session.instance.as_ref()))
        .content_type(HTML)
        .header("Cookie", format!("session={}", session.id.as_ref()))
        .expect_string()
        .send(Event::DisplayNote);

    caps.render.render();
}

pub fn display_note(
    model: &mut Model,
    caps: &Capabilities,
    response: Result<Response<String>, HttpError>,
) {
    if let Ok(mut response) = response {
        model.note = Some(response.take_body().unwrap().into());
        caps.render.render();
        return;
    }

    let session = match model.session.as_ref() {
        Some(session) => session,
        None => return,
    };

    model.note = Some(format!("Failed to fetch note from {}", session.instance.as_ref()).into());

    // TODO: Notify user and return to login screen
    // Send empty buffer to signal erasure of key
    caps.key_value
        .write("session", Vec::new(), Event::SaveSession);
    caps.render.render();
}

#[cfg(test)]
mod note_tests {
    use core::panic;

    use crux_core::testing::AppTester;
    use crux_http::protocol::{HttpResponse, HttpResult};
    use crux_kv::KeyValueOutput;

    use crate::auth::Session;
    use crate::{Effect, Event, Model, Note};

    #[test]
    fn get_note() {
        let instance = "http://localhost:3000";
        let session_id = "sessionid123";

        let app: AppTester<Note, _> = AppTester::default();
        let mut model = Model {
            note: Some("No note available".into()),
            session: Some(Session {
                id: "sessionid123".into(),
                instance: "http://localhost:3000".into(),
            }),
        };

        // Provide test session
        let _ = app.update(
            Event::LoadSession(KeyValueOutput::Read(Some(
                serde_json::to_vec(&Session {
                    id: session_id.into(),
                    instance: instance.into(),
                })
                .unwrap(),
            ))),
            &mut model,
        );

        assert!(model
            .session
            .as_ref()
            .is_some_and(|session| *session.id == *session_id));

        let mut update = app.update(Event::GetNote, &mut model);
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

        let event = res.events.get(0).unwrap().clone();
        let _ = app.update(event, &mut model);

        let view = app.view(&model);
        assert_eq!(view.note, body.into());
    }
}
