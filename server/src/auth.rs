use cfg_if::cfg_if;
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
    // salt: String,
    anonymous: bool,
}

// Implement user auth methods
cfg_if!( if #[cfg(feature = "ssr")] {
    use async_trait::async_trait;
    use axum_session_auth::{Authentication, AuthSession, SessionSurrealPool};
    use crate::settings::LazyNotesSettings;
    use surrealdb::{engine::remote::ws::Client, Surreal};
    use std::fs::{create_dir_all, File};

    impl User {
        pub async fn get(username: String, pool: &Surreal<Client>) -> Option<Self> {
            let sqluser: Option<SqlUser> = pool
                .select(("users", username))
                .await
                .ok()?;

            Some(sqluser?.into_user())
        }
    }

    impl Default for User {
        fn default() -> Self {
            Self {
                username: "Guest".into(),
                password: "".into(),
                anonymous: true,
            }
        }
    }

    #[async_trait]
    impl Authentication<User, String, Surreal<Client>> for User {
        async fn load_user(
            username: String,
            pool: Option<&Surreal<Client>>,
        ) -> Result<User, anyhow::Error> {
            let pool = pool.unwrap();
            User::get(username, &pool)
                .await
                .ok_or_else(|| anyhow::anyhow!("Could not load user!"))
        }

        fn is_authenticated(&self) -> bool {
            !self.anonymous
        }

        fn is_active(&self) -> bool {
            true
        }

        fn is_anonymous(&self) -> bool {
            self.anonymous
        }
    }
});

#[derive(Serialize, Deserialize, Clone)]
pub struct SqlUser {
    pub username: String,
    pub password: String,
}

impl SqlUser {
    pub fn into_user(self) -> User {
        User {
            username: self.username,
            password: self.password,
            anonymous: false,
        }
    }
}

/// API endpoint which handles user signups.
#[server(endpoint = "signup")]
pub async fn signup(
    username: String,
    password: String,
    password_confirmation: String,
    // remember: bool,
) -> Result<(), ServerFnError> {
    let pool: Surreal<Client> = use_context().ok_or_else(|| ServerFnError::new("Pool missing"))?;

    if let Some(_user) = User::get(username.clone(), &pool).await {
        return Err(ServerFnError::new("Username is taken"));
    }

    if password != password_confirmation {
        return Err(ServerFnError::new("Passwords did not match"));
    }

    // Create user directories
    let ln_settings: LazyNotesSettings = expect_context();
    let user_dir = format!("{}/{}", &ln_settings.data_dir, &username);
    let _ = create_dir_all(format!("{}/notes", &user_dir));
    let _ = create_dir_all(format!("{}/resources", &user_dir));
    let _ = File::create_new(format!("{}/notes/index.md", user_dir));

    let _record: Option<SqlUser> = pool
        .create(("users", username.clone()))
        .content(SqlUser { username, password })
        .await
        .map_err(|_| ServerFnError::new("Failed to create user"))?;

    leptos_axum::redirect("/");
    Ok(())
}

/// API endpoint which handles user login.
#[server(endpoint = "login")]
pub async fn login(
    username: String,
    password: String,
    // remember: bool,
) -> Result<(), ServerFnError> {
    let pool: Surreal<Client> = use_context().ok_or_else(|| ServerFnError::new("Pool missing"))?;
    let auth: AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        use_context().ok_or_else(|| ServerFnError::new("Auth session missing"))?;

    // TODO: Handle invalid username inputs
    let user = User::get(username, &pool)
        .await
        .ok_or_else(|| ServerFnError::new("User does not exist"))?;

    if password != user.password {
        return Err(ServerFnError::new("Incorrect password"));
    }

    auth.login_user(user.username.clone());
    leptos_axum::redirect(&format!("/{}/notes/index.md", &user.username));
    Ok(())
}

/// API endpoint to logout the user.
#[server(endpoint = "logout")]
pub async fn logout() -> Result<(), ServerFnError> {
    let auth: AuthSession<User, String, SessionSurrealPool<Client>, Surreal<Client>> =
        use_context().ok_or_else(|| ServerFnError::new("Auth session missing"))?;

    auth.logout_user();
    leptos_axum::redirect("/");
    Ok(())
}

#[cfg(test)]
mod tests {
    // TODO: Make test code cleaner (better way to reset after post-testing)
    // NOTE: Tests requires server running
    use crate::auth::SqlUser;
    // use crate::settings;
    use reqwest;
    use surrealdb::{
        engine::remote::ws::{Client, Ws},
        opt::auth::Namespace,
        Surreal,
    };

    async fn get_db() -> Surreal<Client> {
        // Get database client from test config
        // let ln_config = settings::get_configuration("tests/test_settings.toml")
        //     .expect("Failed to read configuration file");
        // let db_settings = ln_config.database;

        // let db = Surreal::new::<Ws>(db_settings.db_host)
        let db = Surreal::new::<Ws>("localhost:8000")
            .await
            .expect("Failed connecting to database");

        db.use_ns("lazy_notes").use_db("lazy_notes").await.unwrap();

        db.signin(Namespace {
            namespace: "lazy_notes",
            username: "admin",
            password: "debug",
        })
        .await
        .unwrap();

        db
    }

    #[tokio::test]
    #[ignore]
    async fn init() {
        // Init test database
        let db = get_db().await;
        let _: Option<SqlUser> = db
            .create(("users", "login_test"))
            .content(SqlUser {
                username: "login_test".to_string(),
                password: "logintest123".to_string(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_signup() {
        // Test we can signup an account with the api endpoint
        let client = reqwest::Client::new();
        let username = "test";
        let password = "test123";

        let params = [
            ("username", username),
            ("password", password),
            ("password_confirmation", password),
        ];
        let res = client
            .post("http://localhost:3000/api/signup")
            .form(&params)
            .send()
            .await
            .unwrap();

        assert!(res.status().is_success());
    }

    #[tokio::test]
    async fn test_login_logout() {
        // Test we can log into an account with the api endpoint
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();

        // Test Login
        let params = [("username", "login_test"), ("password", "logintest123")];
        let res = client
            .post("http://localhost:3000/api/login")
            .form(&params)
            .send()
            .await
            .unwrap();

        assert!(res.status().is_success());

        // Test Logout
        let res = client
            .post("http://localhost:3000/api/logout")
            .send()
            .await
            .unwrap();

        assert!(res.status().is_success());
    }

    #[tokio::test]
    #[ignore]
    async fn reset() {
        // Reset tests where necessary
        let db = get_db().await;
        let _: Option<SqlUser> = db
            .delete(("users", "test"))
            .await
            .expect("User 'test' in table 'users'");
    }
}
