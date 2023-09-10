use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::exit,
};

use askama::Template;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Router, Server,
};
use sqlx::SqlitePool;
use tower_cookies::{CookieManagerLayer, Cookies};
use tracing::{error, info, warn, Level};
use utils::HtmlTemplate;

use crate::error::FunkyError;

mod assets;
mod auth;
mod error;
mod login;
mod tags;
mod utils;

#[derive(Clone)]
pub struct FunkyState {
    db_pool: SqlitePool,
}

#[derive(Debug)]
enum LoginState {
    None,
    Connected { username: String },
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_state: LoginState,
}

#[axum::debug_handler]
async fn root(
    State(state): State<FunkyState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, FunkyError> {
    let auth_data = cookies
        .get("token")
        .map(|token| auth::verify(token.value()))
        .flatten();

    let index = IndexTemplate {
        login_state: match auth_data {
            Some(data) => {
                let lad = sqlx::query!("SELECT username FROM Lad WHERE id = ?", data.user_id)
                    .fetch_one(&state.db_pool)
                    .await?;
                LoginState::Connected {
                    username: lad.username,
                }
            }
            None => LoginState::None,
        },
    };

    Ok(HtmlTemplate(index))
}

// #[launch]
// async fn launch() -> _ {
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    info!("Logging initialized");

    if let Err(e) = dotenvy::dotenv() {
        warn!(error = ?e, "Unable to load .env file.");
    }

    let Ok(url) = env::var("DATABASE_URL") else {
        error!("DATABASE_URL environment variable not found.");
        exit(1);
    };
    let db_pool = match SqlitePool::connect(&url).await {
        Ok(p) => p,
        Err(e) => {
            error!(error = ?e, "Unable to initialize database pool.");
            error!("The database file must exist before running the app.");
            error!("You can create it with `touch /path/to/file.db`");
            exit(1);
        }
    };
    info!("Database Connection Pool setup");

    // Idk, is it needed ?
    if let Err(e) = sqlx::migrate!().run(&db_pool).await {
        error!(error = ?e, "An error occured when doing database migration.");
    }

    // Building axum app
    let app = Router::new()
        // ROUTES
        // Base routes
        .route("/", get(root))
        .route("/assets/*file", get(assets::serve_asset))
        // Login
        .route("/login", get(login::get_login))
        .route("/login", post(login::login))
        .route("/logout", get(login::logout))
        // Tags
        .route("/tags", get(tags::get_tags))
        //TODO: Handler for overlapping routes
        .route(
            "/tags/new",
            get(tags::get_tags_form).post(tags::post_new_tag),
        )
        .layer(CookieManagerLayer::new())
        .with_state(FunkyState { db_pool });

    let port = match env::var("PORT") {
        Ok(p) => p.parse().unwrap_or(8000),
        Err(_) => {
            info!("PORT not set, using default port 8000");
            8000
        }
    };
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    info!("Listening on {}", addr);

    Ok(Server::bind(&addr).serve(app.into_make_service()).await?)
}
