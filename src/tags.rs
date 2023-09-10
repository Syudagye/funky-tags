use std::collections::HashMap;

use askama::Template;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Form,
};
use serde::Deserialize;
use tower_cookies::Cookies;
use tracing::error;

use crate::{
    auth,
    error::FunkyError,
    utils::{FormMessage, HtmlTemplate},
    FunkyState,
};

#[derive(Template)]
#[template(path = "tags.html")]
pub struct TagsTemplate {
    tags: Vec<Gamertag>,
}

pub struct Gamertag {
    username: String,
    ladname: String,
    gamename: String,
}

#[axum::debug_handler]
pub async fn get_tags(State(state): State<FunkyState>) -> Result<impl IntoResponse, FunkyError> {
    let tags = sqlx::query_as!(
        Gamertag,
        "
            SELECT
                Gamertag.username as username,
                Lad.username as ladname,
                Game.name as gamename
            FROM Gamertag
            JOIN Lad on Lad.id = Gamertag.poster
            JOIN Game on Game.id = Gamertag.game
        "
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(HtmlTemplate(TagsTemplate { tags }))
}

#[derive(Template)]
#[template(path = "tagsForm.html")]
pub struct NewTagFormTemplate;

#[derive(Template)]
#[template(path = "gameSelect.html")]
pub struct NewGameTemplate {
    new: bool,
    games: Vec<Game>,
}

pub struct Game {
    name: String,
    id: i64,
    selected: bool,
}

#[axum::debug_handler]
pub async fn get_tags_form(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<FunkyState>,
) -> Result<impl IntoResponse, FunkyError> {
    // If there is no game param, serve the form
    let Some(game) = params.get("game") else {
        let form = NewTagFormTemplate;
        return Ok(HtmlTemplate(form).into_response());
    };

    let newgame = match game.as_str() {
        "new" => NewGameTemplate {
            new: true,
            games: vec![],
        },
        _ => NewGameTemplate {
            new: false,
            games: sqlx::query!("SELECT * FROM Game")
                .fetch_all(&state.db_pool)
                .await?
                .into_iter()
                .map(|row| Game {
                    name: row.name,
                    id: row.id,
                    selected: &row.id.to_string() == game,
                })
                .collect(),
        },
    };

    Ok(HtmlTemplate(newgame).into_response())
}

#[derive(Deserialize)]
pub struct NewGamertag {
    gamertag: String,
    game: Option<i64>,
    newgame: Option<String>,
}

#[axum::debug_handler]
pub async fn post_new_tag(
    State(state): State<FunkyState>,
    cookies: Cookies,
    Form(body): Form<NewGamertag>,
) -> impl IntoResponse {
    let Some(auth_data) = cookies
        .get("token")
        .map(|token| auth::verify(token.value()))
        .flatten()
    else {
        return FormMessage::Err("You are not authorized to add new tags :p");
    };

    let game_id = match (body.newgame, body.game) {
        (Some(new_game), None) => {
            match sqlx::query!("INSERT INTO Game (name) VALUES (?) RETURNING id", new_game)
                .fetch_one(&state.db_pool)
                .await
            {
                Ok(q) => q.id,
                Err(e) => {
                    error!(error = ?e, "An error occured when issueing a query.");
                    return FormMessage::Err("An error occured :c");
                }
            }
        }
        (None, Some(game)) => game,
        _ => return FormMessage::Err("Malformed request"),
    };

    if let Err(e) = sqlx::query!(
        "INSERT INTO Gamertag (username, poster, game) VALUES (?, ?, ?)",
        body.gamertag,
        auth_data.user_id,
        game_id
    )
    .execute(&state.db_pool)
    .await
    {
        error!(error = ?e, "An error occured when issueing a query.");
        return FormMessage::Err("An error occured :c");
    }

    FormMessage::Ok("Name added !")
}
