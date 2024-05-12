use crux_http::{http::mime::HTML, HttpError, Response};

use crate::parser::HtmlParseResult;
use crate::{Capabilities, Event, Model};

pub fn get_note(model: &mut Model, caps: &Capabilities, path: &str) {
    let session = match model.session.as_ref() {
        Some(session) => session,
        None => return,
    };

    caps.http
        .get(format!(
            "{}/{}/notes/{path}",
            session.instance.as_ref(),
            session.username.as_ref()
        ))
        .content_type(HTML)
        .header("Cookie", format!("session={}", session.id.as_ref()))
        .expect_string()
        .send(Event::ParseNote);

    caps.render.render();
}

pub fn parse_note(caps: &Capabilities, response: Result<Response<String>, HttpError>) {
    if let Ok(mut response) = response {
        caps.html_parser.parse_html(
            &response.take_body().unwrap_or_default(),
            Event::DisplayNote,
        );
        return;
    }

    // TODO: Notify user and return to login screen with error message
    // If bad response, assume bad session and reset local key
    // Send empty buffer to signal erasure of key
    caps.key_value
        .write("session", Vec::new(), Event::SaveSession);
    caps.render.render();
}

pub fn display_note(model: &mut Model, caps: &Capabilities, result: HtmlParseResult) {
    model.note = result.nodes;
    caps.render.render()
}

#[cfg(test)]
mod note_tests {
    use core::panic;

    use crux_core::testing::AppTester;
    use crux_http::protocol::{HttpResponse, HttpResult};
    use crux_kv::KeyValueOutput;

    use crate::auth::Session;
    use crate::parser::HtmlNode;
    use crate::{Effect, Event, Model, Note};

    #[test]
    fn get_note() {
        let instance = "http://localhost:3000";
        let session_id = "sessionid123";
        let username = "login_test123";

        let app: AppTester<Note, _> = AppTester::default();
        let mut model = Model {
            note: vec![],
            session: Some(Session {
                id: session_id.into(),
                instance: instance.into(),
                username: username.into(),
            }),
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
        let body_parsed = vec![HtmlNode {
            tag: "h1".into(),
            body: Some("Success".into()),
        }];

        let res = app
            .resolve(req, HttpResult::Ok(HttpResponse::ok().body(body).build()))
            .unwrap();

        let parse_event = res.events.get(0).unwrap().clone();
        let update = app.update(parse_event, &mut model);

        let display_event = update.events.get(0).unwrap().clone();
        let _ = app.update(display_event, &mut model);

        let view = app.view(&model);
        assert_eq!(view.note, body_parsed);
    }
}
