use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
use leptos::logging;
use serde::Deserialize;
use std::fs::read_to_string;

// Define toml layout
#[derive(Debug, Deserialize, Clone)]
struct LazyNotesToml {
    settings: LazyNotesSettings
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LazyNotesSettings {
    // TODO: Add field for db type
    pub db_host: Option<String>,
    pub notes_dir: String,
    pub resources_dir: String,
}

pub fn get_configuration(path: Option<String>) -> Option<LazyNotesSettings> {
    enum Errors {
        IoError,
        ParseError,
    }

    let config_str = match path {
        Some(config_path) => read_to_string(config_path),
        None => read_to_string("settings.toml"),
    };

    config_str
        .map_err(|err| {
            logging::error!("Failed to read configuration file: {err}");
            Errors::IoError
         })
        .and_then(|config| {
            toml::from_str::<LazyNotesToml>(&config).map_err(|err| {
                logging::error!("Failed to parse toml configuration: {err}");
                Errors::ParseError
            })
        })
        .map(|ln_toml| { ln_toml.settings })
        .ok()

    // TODO: Parse env variables
}

mod tests {
    #[allow(dead_code)]
    fn get_settings_file() -> &'static str {
        "tests/test_settings.toml"
    }

    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    #[test]
    fn can_parse_configuration() {
        use crate::settings::get_configuration;
        assert_ne!(get_configuration(Some(get_settings_file().to_string())), None);
    }

    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    #[test]
    fn configuration_correct() {
        use crate::settings::get_configuration;
        let ln_settings = get_configuration(Some(get_settings_file().to_string())).unwrap();
        assert_eq!(ln_settings.db_host, None);
        assert_eq!(ln_settings.notes_dir, "tests/notes");
        assert_eq!(ln_settings.resources_dir, "tests/resources");
    }
}
}}
