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

#[derive(Template)]
#[template(path = "loginForm.html")]
struct LoginFormTemplate;

#[get("/login")]
async fn get_login() -> Result<(ContentType, String), FunkyError> {
    let form = LoginFormTemplate;
    Ok(form.render().map(|s| (ContentType::HTML, s))?)
}

#[derive(Debug, FromForm)]
struct LoginForm<'a> {
    username: &'a str,
    passwd: &'a str,
}

#[post("/login", data = "<creds>")]
async fn login(
    creds: Form<LoginForm<'_>>,
    pool: &State<SqlitePool>,
    cookies: &CookieJar<'_>,
) -> Result<(ContentType, String), (ContentType, String)> {
    let error = Err((
        ContentType::HTML,
        String::from(
            r#"<div class="login-msg login-msg--error">Your are not allowed here, go away >:(</span>"#,
        ),
    ));

    let inner_form = creds.into_inner();
    trace!(form = ?inner_form, "Login requested.");

    let lad = match sqlx::query!("SELECT * FROM Lad WHERE username = ?", inner_form.username)
        .fetch_one(pool.inner())
        .await
    {
        Ok(lad) => lad,
        Err(e) => {
            trace!(error = ?e, "Requested login credentials not found in database, rejecting login.");
            return error;
        }
    };
    trace!(?lad, "Succesfully queried user credentials from database");

    let form_passwd_hash = utils::sha256_str(inner_form.passwd);

    if form_passwd_hash != lad.passwd_hash {
        trace!("Invalid password, rejecting login.");
        return error;
    }

    cookies.add(Cookie::new(
        "token",
        auth::sign(TokenData { user_id: lad.id }),
    ));

    Err((
        ContentType::HTML,
        String::from(
            r#"<div class="login-msg login-msg--success" hx-on:htmx:load="location.reload()">Login successful :D</div>"#,
        ),
    ))
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> (ContentType, String) {
    cookies.remove(Cookie::named("token"));
    (
        ContentType::HTML,
        String::from(r#"<div hx-on:htmx:load="location.reload()"></div>"#),
    )
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
        .mount("/", routes![root, get_login, login, logout])
        .mount("/assets", FileServer::from(relative!("assets")))
}
