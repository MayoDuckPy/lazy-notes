use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
use leptos::logging;
use serde::Deserialize;
use std::fs::read_to_string;

// Define toml layout
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct LazyNotesConfiguration {
    pub settings: LazyNotesSettings,
    pub database: DatabaseSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LazyNotesSettings {
    // TODO: Add field for db type
    pub data_dir: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DatabaseSettings {
    pub db_host: String,
    pub database: String,
    pub namespace: String,
    pub username: String,
    pub password: String,
}

pub fn get_configuration(path: Option<String>) -> Option<LazyNotesConfiguration> {
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
            toml::from_str::<LazyNotesConfiguration>(&config).map_err(|err| {
                logging::error!("Failed to parse toml configuration: {err}");
                Errors::ParseError
            })
        })
        .ok()

    // TODO: Parse env variables
}

#[cfg(feature = "ssr")]
#[cfg(test)]
mod tests {
    #[allow(dead_code)]
    fn get_settings_file() -> &'static str {
        "tests/test_settings.toml"
    }

    #[test]
    fn can_parse_configuration() {
        use crate::settings::get_configuration;
        assert_ne!(get_configuration(Some(get_settings_file().to_string())), None);
    }

    #[test]
    fn configuration_correct() {
        use crate::settings::get_configuration;
        let ln_config = get_configuration(Some(get_settings_file().to_string())).unwrap();
        assert_eq!(ln_config.database.db_host, "localhost:8000");
        assert_eq!(ln_config.settings.data_dir, "tests/notes");
    }
}
}}
