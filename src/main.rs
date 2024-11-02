use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing, Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file.");
    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("localhost:8080".to_owned());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    let db_pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres.");

    let listener = TcpListener::bind(server_address)
        .await
        .expect("Failed to bind to address.");

    let router = Router::new()
        .route("/", routing::get(|| async { "Hello, World!" }))
        .route("/tasks", routing::get(get_tasks).post(create_task))
        .route("/tasks/:id", routing::put(update_task).delete(delete_task))
        .with_state(db_pool);

    axum::serve(listener, router)
        .await
        .expect("Failed to start server.");
}

#[derive(Serialize)]
struct TaskRow {
    id: i32,
    name: String,
    priority: Option<i32>,
}

async fn get_tasks(
    State(db_pool): State<PgPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    return sqlx::query_as!(TaskRow, "SELECT * FROM tasks ORDER BY id")
        .fetch_all(&db_pool)
        .await
        .map(|rows| {
            (
                StatusCode::OK,
                json!({ "success": true, "data": rows }).to_string(),
            )
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": e.to_string() }).to_string(),
            )
        });
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    name: String,
    priority: Option<i32>,
}

#[derive(Serialize)]
struct CreateTaskRow {
    id: i32,
}

async fn create_task(
    State(db_pool): State<PgPool>,
    Json(task): Json<CreateTaskRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    return sqlx::query_as!(
        CreateTaskRow,
        "INSERT INTO tasks (name, priority) VALUES ($1, $2) RETURNING id",
        task.name,
        task.priority
    )
    .fetch_one(&db_pool)
    .await
    .map(|row| {
        (
            StatusCode::OK,
            json!({ "success": true, "data": row }).to_string(),
        )
    })
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success": false, "message": e.to_string() }).to_string(),
        )
    });
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    name: Option<String>,
    priority: Option<i32>,
}

async fn update_task(
    State(db_pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(task): Json<UpdateTaskRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    return sqlx::query!(
        "UPDATE tasks SET name = $2, priority = $3 WHERE id = $1",
        id,
        task.name,
        task.priority
    )
    .execute(&db_pool)
    .await
    .map(|_| (StatusCode::OK, json!({ "success": true }).to_string()))
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success": false, "message": e.to_string() }).to_string(),
        )
    });
}

async fn delete_task(
    State(db_pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    return sqlx::query!("DELETE FROM tasks WHERE id = $1", id)
        .execute(&db_pool)
        .await
        .map(|_| (StatusCode::OK, json!({ "success": true }).to_string()))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": e.to_string() }).to_string(),
            )
        });
}
