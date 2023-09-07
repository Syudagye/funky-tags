use askama::Template;
use rocket::{
    form::Form,
    get,
    http::{ContentType, Cookie, CookieJar},
    post, FromForm, State,
};
use sqlx::SqlitePool;
use tracing::trace;

use crate::{
    auth::{self, TokenData},
    error::FunkyError,
    utils,
};

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
    let inner_form = creds.into_inner();
    trace!(form = ?inner_form, "Login requested.");

    let lad = sqlx::query!("SELECT * FROM Lad WHERE username = ?", inner_form.username)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| {
            trace!(error = ?e, "Requested login credentials not found in database, rejecting login.");
            utils::build_form_msg(Err("Your are not allowed here, go away >:("))
        })?;

    trace!(?lad, "Succesfully queried user credentials from database");

    let form_passwd_hash = utils::sha256_str(inner_form.passwd);

    if form_passwd_hash != lad.passwd_hash {
        trace!("Invalid password, rejecting login.");
        return Err(utils::build_form_msg(Err("Your are not allowed here, go away >:(")));
    }

    cookies.add(Cookie::new(
        "token",
        auth::sign(TokenData { user_id: lad.id }),
    ));

    Ok(utils::build_form_msg(Ok("Login successful :D")))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> (ContentType, String) {
    cookies.remove(Cookie::named("token"));
    (
        ContentType::HTML,
        String::from(r#"<div hx-on:htmx:load="location.reload()"></div>"#),
    )
}
