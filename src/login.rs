use askama::Template;
use axum::{extract::State, response::{IntoResponse, Html}, Form};
use serde::Deserialize;
use tower_cookies::{Cookies, Cookie};
use tracing::trace;

use crate::{
    auth::{self, TokenData},
    utils::{self, FormMessage, HtmlTemplate},
    FunkyState,
};

#[derive(Template)]
#[template(path = "loginForm.html")]
struct LoginFormTemplate;

#[axum::debug_handler]
pub async fn get_login() -> impl IntoResponse {
    HtmlTemplate(LoginFormTemplate)
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    username: String,
    passwd: String,
}

#[axum::debug_handler]
pub async fn login(
    State(state): State<FunkyState>,
    cookies: Cookies,
    Form(creds): Form<LoginForm>,
) -> impl IntoResponse {
    trace!(?creds, "Login requested.");
    let err = FormMessage::Err("Your are not allowed here, go away >:(");

    let lad = match sqlx::query!("SELECT * FROM Lad WHERE username = ?", creds.username)
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(l) => l,
        Err(e) => {
            trace!(error = ?e, "Requested login credentials not found in database, rejecting login.");
            return err;
        }
    };

    trace!(?lad, "Succesfully queried user credentials from database");

    let form_passwd_hash = utils::sha256_str(creds.passwd);

    if form_passwd_hash != lad.passwd_hash {
        trace!("Invalid password, rejecting login.");
        return err;
    }

    cookies.add(Cookie::new(
        "token",
        auth::sign(TokenData { user_id: lad.id }),
    ));

    FormMessage::Ok("Login successful :D")
}

#[axum::debug_handler]
pub async fn logout(cookies: Cookies) -> impl IntoResponse {
    cookies.remove(Cookie::named("token"));
    Html(r#"<div hx-on:htmx:load="location.reload()"></div>"#)
}
