use askama::Template;
use rocket::{
    form::Form,
    get,
    http::{ContentType, CookieJar},
    post, FromForm, State,
};
use sqlx::SqlitePool;

use crate::{auth, error::FunkyError, utils};

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

#[get("/tags")]
pub async fn get_tags(pool: &State<SqlitePool>) -> Result<(ContentType, String), FunkyError> {
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
    .fetch_all(&**pool)
    .await?;

    let html = TagsTemplate { tags };

    Ok(html.render().map(|s| (ContentType::HTML, s))?)
}

#[derive(Template)]
#[template(path = "tagsForm.html")]
pub struct NewTagFormTemplate;

#[get("/tags/new")]
pub async fn get_tags_form() -> Result<(ContentType, String), FunkyError> {
    let form = NewTagFormTemplate;

    Ok(form.render().map(|s| (ContentType::HTML, s))?)
}

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

#[get("/tags/new?<game>")]
pub async fn get_new_game_field(
    pool: &State<SqlitePool>,
    game: &str,
) -> Result<(ContentType, String), FunkyError> {
    let newgame = match game {
        "new" => NewGameTemplate {
            new: true,
            games: vec![],
        },
        _ => NewGameTemplate {
            new: false,
            games: sqlx::query!("SELECT * FROM Game")
                .fetch_all(&**pool)
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

    Ok(newgame.render().map(|s| (ContentType::HTML, s))?)
}

#[derive(FromForm)]
pub struct NewGamertag<'a> {
    gamertag: &'a str,
    game: Option<i64>,
    newgame: Option<&'a str>,
}

#[post("/tags/new", data = "<body>")]
pub async fn post_new_tag(
    pool: &State<SqlitePool>,
    cookies: &CookieJar<'_>,
    body: Form<NewGamertag<'_>>,
) -> Result<(ContentType, String), (ContentType, String)> {
    let auth_data = cookies
        .get("token")
        .map(|token| auth::verify(token.value()))
        .flatten()
        .ok_or(utils::build_form_msg(Err(
            "You are not authorized to add new tags :p",
        )))?;

    let game_id = match (body.newgame, body.game) {
        (Some(new_game), None) => {
            sqlx::query!("INSERT INTO Game (name) VALUES (?) RETURNING id", new_game)
                .fetch_one(&**pool)
                .await
                .map_err(|_| utils::build_form_msg(Err("An error occured :c")))?
                .id
        }
        (None, Some(game)) => game,
        _ => Err(utils::build_form_msg(Err("Malformed request")))?,
    };

    sqlx::query!(
        "INSERT INTO Gamertag (username, poster, game) VALUES (?, ?, ?)",
        body.gamertag,
        auth_data.user_id,
        game_id
    )
    .execute(&**pool)
    .await
    .map_err(|_| utils::build_form_msg(Err("An error occured :c")))?;

    Ok(utils::build_form_msg(Ok("Name Added !")))
}
