use cfg_if::cfg_if;
// pub mod api;
pub mod app;
pub mod auth;
pub mod settings;
pub mod state;
// pub mod error_template;
// pub mod fileserv;

cfg_if! { if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use log::Level::Debug;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
        // initializes logging using the `log` crate
        _ = console_log::init_with_level(Debug);
        console_error_panic_hook::set_once();

        leptos_dom::HydrationCtx::stop_hydrating();
    }
}}
