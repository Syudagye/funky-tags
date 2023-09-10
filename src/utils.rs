use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use sha2::{Digest, Sha256};

/// Returns a String hash of the given bytes
pub fn sha256_str(bytes: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Form feedback message
pub enum FormMessage<'a> {
    Ok(&'a str),
    Err(&'a str),
}

impl<'a> IntoResponse for FormMessage<'a> {
    fn into_response(self) -> Response {
        let (attrs, msg) = match self {
            FormMessage::Ok(msg) => (
                r#"class="form-msg form-msg--success" hx-on:htmx:load="location.reload()""#,
                msg,
            ),
            FormMessage::Err(msg) => (r#"class="form-msg form-msg--error""#, msg),
        };
        Html(format!("<div {}>{}</div>", attrs, msg)).into_response()
    }
}

/// Wrapper for html templates
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
