use askama::Template;
use rocket::{http::{ContentType, CookieJar, Cookie}, FromForm, State, get, post, form::Form};
use sqlx::SqlitePool;
use tracing::trace;

use crate::{error::FunkyError, utils, auth::{self, TokenData}};

#[derive(Template)]
#[template(path = "loginForm.html")]
struct LoginFormTemplate;

#[get("/login")]
pub fn get_login() -> Result<(ContentType, String), FunkyError> {
    let form = LoginFormTemplate;
    Ok(form.render().map(|s| (ContentType::HTML, s))?)
}

#[derive(Debug, FromForm)]
pub struct LoginForm<'a> {
    username: &'a str,
    passwd: &'a str,
}

#[post("/login", data = "<creds>")]
pub async fn login(
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
pub async fn logout(cookies: &CookieJar<'_>) -> (ContentType, String) {
    cookies.remove(Cookie::named("token"));
    (
        ContentType::HTML,
        String::from(r#"<div hx-on:htmx:load="location.reload()"></div>"#),
    )
}
