use cfg_if::cfg_if;

cfg_if!( if #[cfg(feature = "ssr")] {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        response::{IntoResponse, Response},
        extract::{Path, State}, Router
    };
    use axum_session::{SessionConfig, SessionLayer, SessionStore};
    use axum_session_auth::{AuthConfig, AuthSession, AuthSessionLayer, SessionSurrealPool};
    use leptos::logging::log;
    use leptos::*;
    use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
    use log::Level::Error;
    use std::env;
    use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Namespace, Surreal};
    use chrono::Duration;
    use tower::util::ServiceExt;
    use tower_http::services::ServeDir;

    use lazy_notes::app::*;
    use lazy_notes::auth::User;
    use lazy_notes::settings;
    use lazy_notes::state::AppState;
});

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    // use lazy_notes::api::api_routes;

    simple_logger::init_with_level(Error).expect("Couldn't initialize logging");

    // Get Lazy Notes configuration
    let ln_config = settings::get_configuration(env::var("LN_SETTINGS_FILE").ok())
        .expect("Failed to read configuration file");
    let ln_settings = ln_config.settings;
    let db_settings = ln_config.database;

    // Setup SurrealDB
    let db = Surreal::new::<Ws>(db_settings.db_host) //.expect("No DB host specified"))
        .await
        .expect("Failed connecting to database");
    db.use_ns("lazy_notes").use_db("lazy_notes").await.unwrap();
    db.signin(Namespace {
        namespace: "lazy_notes",
        username: &db_settings.username,
        password: &db_settings.password,
    })
    .await
    .unwrap();

    // Setup auth
    let pool = SessionSurrealPool::<Client>::new(db.clone());
    let session_config = SessionConfig::default().with_table_name("sessions");
    let auth_config = AuthConfig::<String>::default()
        .with_max_age(Duration::try_weeks(2).expect("Overflow on max session age"));
    let session_store =
        SessionStore::<SessionSurrealPool<Client>>::new(Some(pool.clone().into()), session_config)
            .await
            .unwrap();

    // Get env values for leptos
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let root = leptos_options.site_root.as_str();
    let routes = generate_route_list(App);

    let app = Router::new()
        .nest_service("/pkg", ServeDir::new(format!("{root}/pkg")))
        .nest_service("/scripts", ServeDir::new(format!("{root}/scripts")))
        .nest_service("/icons", ServeDir::new(format!("{root}/icons")))
        .route("/:user/resources/*file", get(note_resource_handler))
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes.clone(), get(leptos_routes_handler))
        // .fallback(file_and_error_handler)
        .layer(
            AuthSessionLayer::<User, String, SessionSurrealPool<Client>, Surreal<Client>>::new(
                Some(db.clone()),
            )
            .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store))
        .with_state(AppState {
            leptos_options,
            settings: ln_settings,
            pool: db,
            routes,
        });
    // .nest("/api", api_routes());

    log!("Listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}

#[cfg(feature = "ssr")]
async fn leptos_routes_handler(
    auth_session: AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>>,
    State(app_state): State<AppState>,
    req: Request<Body>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
            provide_context(app_state.settings.clone());
        },
        App,
    );

    handler(req).await.into_response()
}

#[cfg(feature = "ssr")]
async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>>,
    _path: Path<String>,
    request: Request<Body>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
            provide_context(app_state.settings.clone());
        },
        request,
    )
    .await
}

#[cfg(feature = "ssr")]
async fn note_resource_handler(
    auth: AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>>,
    Path((username, _file)): Path<(String, String)>,
    State(state): State<AppState>,
    req: Request<Body>,
) -> Response {
    let root = state.settings.data_dir;

    if !auth.is_authenticated()
        || auth.current_user.expect("User is authenticated").username != username
    {
        return (
            StatusCode::UNAUTHORIZED,
            "Resource requires permission to view",
        )
            .into_response();
    }

    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )
            .into_response(),
    }
}
