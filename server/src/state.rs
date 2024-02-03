use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::extract::FromRef;
        use crate::settings::LazyNotesSettings;
        use leptos::LeptosOptions;
        use leptos_router::RouteListing;
        use surrealdb::{engine::remote::ws::Client, Surreal};

        #[derive(FromRef, Debug, Clone)]
        pub struct AppState {
            pub leptos_options: LeptosOptions,
            pub settings: LazyNotesSettings,
            pub pool: Surreal<Client>,
            pub routes: Vec<RouteListing>,
        }
    }
}
