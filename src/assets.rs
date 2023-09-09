//! Yes i'm going to include every assets in the final binary, because fuck you

use rocket::{http::ContentType, get};

const CSS_BUNDLE: &'static str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

#[get("/assets/style.css")]
pub fn serve_css() -> (ContentType, &'static str) {
    (ContentType::CSS, CSS_BUNDLE)
}
