use rocket::http::ContentType;
use sha2::{Digest, Sha256};

/// Returns a String hash of the given bytes
pub fn sha256_str(bytes: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Build message reply for form validation
pub fn build_form_msg(msg: Result<&str, &str>) -> (ContentType, String) {
    let (attrs, msg) = match msg {
        Ok(msg) => (
            r#"class="form-msg form-msg--success" hx-on:htmx:load="location.reload()""#,
            msg,
        ),
        Err(msg) => (r#"class="form-msg form-msg--error""#, msg),
    };
    (
        ContentType::HTML,
        String::from(format!(r#"<div {}>{}</span>"#, attrs, msg)),
    )
}
