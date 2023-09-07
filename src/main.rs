use std::{env, process::exit};

use askama::Template;
use rocket::{
    form::Form,
    fs::{relative, FileServer},
    get,
    http::{ContentType, Cookie, CookieJar},
    launch, post, routes, FromForm, State,
};
use sqlx::SqlitePool;
use tracing::{error, info, trace, warn};

use crate::{auth::TokenData, error::FunkyError};

mod auth;
mod error;
mod login;
mod tags;
mod utils;

#[derive(Debug)]
enum LoginState {
    None,
    Connected { username: String },
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_state: LoginState,
}

#[get("/")]
async fn root(
    pool: &State<SqlitePool>,
    cookies: &CookieJar<'_>,
) -> Result<(ContentType, String), FunkyError> {
    let auth_data = cookies
        .get("token")
        .map(|token| auth::verify(token.value()))
        .flatten();

    let index = IndexTemplate {
        login_state: match auth_data {
            Some(data) => {
                let lad = sqlx::query!("SELECT username FROM Lad WHERE id = ?", data.user_id)
                    .fetch_one(pool.inner())
                    .await?;
                LoginState::Connected {
                    username: lad.username,
                }
            }
            None => LoginState::None,
        },
    };

    Ok(index.render().map(|s| (ContentType::HTML, s))?)
}

#[launch]
async fn launch() -> _ {
    tracing_subscriber::fmt().finish();
    info!("Logging initialized");

    if let Err(e) = dotenvy::dotenv() {
        warn!(error = ?e, "Unable to load .env file.");
    }

    let Ok(url) = env::var("DATABASE_URL") else {
        error!("DATABASE_URL environment variable not found.");
        exit(1);
    };
    let pool = match SqlitePool::connect(&url).await {
        Ok(p) => p,
        Err(e) => {
            error!(error = ?e, "Unable to initialize database pool.");
            exit(1);
        }
    };
    info!("Database Connection Pool setup");

    // Idk, is it needed ?
    if let Err(e) = sqlx::migrate!().run(&pool).await {
        error!(error = ?e, "An error occured when doing database migration.");
    }

    rocket::build()
        .manage(pool)
        .mount(
            "/",
            routes![
                root,
                login::get_login,
                login::login,
                login::logout,
                tags::get_tags
            ],
        )
        .mount("/assets", FileServer::from(relative!("assets")))
}
