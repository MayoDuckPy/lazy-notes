use crux_http::{HttpError, Response};
use serde::{Deserialize, Serialize};

use crate::{Capabilities, Event, Model};

const LOGIN_API: &str = "/api/login";
const _SIGNUP_API: &str = "/api/signup";

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: Box<str>,
    pub instance: Box<str>,
}

pub fn login(caps: &Capabilities, instance: Box<str>, username: Box<str>, password: Box<str>) {
    let instance: Box<str> = instance.strip_suffix("/").unwrap_or(&instance).into();

    caps.http
        .post(format!("{instance}{LOGIN_API}"))
        .body_string(format!("username={username}&password={password}"))
        .send(|res| Event::HandleLogin(res, instance));
}

pub fn handle_login(
    model: &mut Model,
    caps: &Capabilities,
    response: Result<Response<Vec<u8>>, HttpError>,
    instance: Box<str>,
) {
    let response = match response {
        Ok(response) => response,
        Err(_e) => {
            return;
        }
    };

    if response.status() != 200 {
        return;
    }

    // Parse session cookie from response
    let session = response
        .header("Set-Cookie")
        .map(|headers| {
            headers
                .iter()
                .map(|cookie| {
                    cookie
                        .as_str()
                        .split_once(";")
                        .unwrap_or_else(|| (cookie.as_str(), ""))
                })
                .map(|(cookie, _)| cookie)
                .flat_map(|cookie| cookie.split_once("="))
                .filter(|&(key, _)| key == "session")
                .map(|(_, val)| val)
                .next()
        })
        .flatten();

    if let Some(session) = session {
        let session = Session {
            id: session.into(),
            instance,
        };

        caps.key_value.write(
            "session",
            serde_json::to_vec(&session).unwrap(),
            Event::SaveSession,
        );

        model.session = Some(session);
        caps.render.render();
    }
    // TODO: Handle else
}

#[cfg(test)]
mod auth_tests {
    use crux_core::testing::AppTester;
    use crux_http::protocol::{HttpRequest, HttpResponse, HttpResult};
    use crux_kv::KeyValueOutput;

    use crate::auth::Session;
    use crate::{Effect, Event, Model, Note};

    #[test]
    fn login_sessionless() {
        let instance = "http://localhost:3000";
        let session_id = "sessionid123";

        let app: AppTester<Note, _> = AppTester::default();
        let mut model = Model {
            note: vec![],
            session: None,
        };

        // Provide login credentials
        let mut update = app.update(
            Event::Login(instance.into(), "login_test".into(), "logintest123".into()),
            &mut model,
        );
        let req = match update.effects_mut().next().unwrap() {
            Effect::Http(req) => req,
            _ => panic!("Unexpected effect from event"),
        };

        // Make login HTTP request
        assert_eq!(
            req.operation,
            HttpRequest::post(format!("{instance}/api/login"))
                .header("content-type", "text/plain;charset=utf-8")
                .body("username=login_test&password=logintest123")
                .build()
        );

        let res = app
            .resolve(
                req,
                HttpResult::Ok(
                    HttpResponse::ok()
                        .header("Set-Cookie", format!("session={session_id}"))
                        .build(),
                ),
            )
            .unwrap();

        // Run login logic which takes HTTP response and saves session id
        let handle_login_event = res.events.get(0).unwrap().clone();
        let mut update = app.update(handle_login_event, &mut model);
        let req = match update.effects_mut().next().unwrap() {
            Effect::KeyValue(req) => req,
            _ => panic!("Unexpected effect from event"),
        };
        let _ = app.resolve(req, KeyValueOutput::Write(true));

        assert!(model
            .session
            .as_ref()
            .is_some_and(|session| *session.id == *session_id && *session.instance == *instance));
    }

    #[test]
    fn login_with_session() {
        let instance = "http://localhost:3000";
        let session_id = "sessionid123";

        let app: AppTester<Note, _> = AppTester::default();
        let mut model = Model {
            note: vec![],
            session: None,
        };

        // Response with success to load session
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
            .is_some_and(|session| *session.id == *session_id && *session.instance == *instance));
    }
}
