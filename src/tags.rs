use askama::Template;
use rocket::{
    get,
    http::{ContentType, CookieJar},
    State,
};
use sqlx::SqlitePool;

use crate::error::FunkyError;

#[derive(Template)]
#[template(path = "tags.html")]
pub struct TagsTemplate {
    tags: Vec<Gamertag>
}

pub struct Gamertag {
    username: String,
    ladname: String,
    gamename: String
}

#[get("/tags")]
pub async fn get_tags(pool: &State<SqlitePool>) -> Result<(ContentType, String), FunkyError> {
    let tags = sqlx::query_as!(Gamertag, "
            SELECT
                Gamertag.username as username,
                Lad.username as ladname,
                Game.name as gamename
            FROM Gamertag
            JOIN Lad on Lad.id = Gamertag.poster
            JOIN Game on Game.id = Gamertag.game
        ")
        .fetch_all(&**pool)
        .await?;

    let html = TagsTemplate {
        tags
    };

    Ok(html.render().map(|s| (ContentType::HTML, s))?)
}
