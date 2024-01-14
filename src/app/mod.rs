use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    serve, Router,
};
use sqlx::sqlite::SqlitePool;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::transactions::{list_transactions, Transaction};

#[derive(Clone)]
struct AppState {
    pub db: SqlitePool,
}

pub async fn run_app(db: SqlitePool) -> anyhow::Result<()> {
    let router = Router::new()
        .route("/", get(index))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(AppState { db });

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, router).await?;
    Ok(())
}

#[derive(Template)]
#[template(path = "transactions.html")]
struct TransactionsTemplate {
    pub transactions: Vec<Transaction>,
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let transactions = list_transactions(&state.db).await.expect("query failed");
    let template = TransactionsTemplate { transactions };
    HtmlTemplate(template)
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
