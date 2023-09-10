//! Yes i'm going to include every assets in the final binary, because fuck you

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[axum::debug_handler]
pub async fn serve_asset(Path(file): Path<String>) -> impl IntoResponse {
    StaticAsset(file)
}

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

pub struct StaticAsset<T>(pub T);

impl<T> IntoResponse for StaticAsset<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Assets::get(&path) {
            Some(file) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}
