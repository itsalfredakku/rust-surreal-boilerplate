pub mod todos_router {
    use crate::data::repositories::todos_repository::TodosRepository;
    use crate::db::Database;
    use crate::data::models::todo::{CreateTodo, Todo, UpdateTodo};
    use axum::extract::Path;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::{
        routing::{get, post},
        Extension, Json, Router,
    };
    use chrono::Local;
    use std::sync::Arc;

    pub fn router() -> Router {
        Router::new()
            .route("/", post(create_todo).get(get_all_todos))
            .route(
                "/:id",
                get(get_todo_by_id).put(update_todo).delete(delete_todo),
            )
            .route("/title/:title", get(get_todo_by_title))
    }

    pub async fn get_all_todos(Extension(db): Extension<Arc<Database>>) -> impl IntoResponse {
        let repository = TodosRepository::new(db);

        let todos = repository.get_all().await.unwrap_or_default();
        let json_response = serde_json::json!({
            "status": "success",
            "count": todos.len(),
            "todos": todos,
        });

        Json(json_response)
    }

    pub async fn get_todo_by_id(
        Extension(db): Extension<Arc<Database>>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
        let repository = TodosRepository::new(db);
        match repository.get_by_id(id.clone()).await {
            Ok(todo) => Ok((StatusCode::OK, Json(todo))),
            Err(_) => Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Todo with ID: {} not found", id)
                })),
            )),
        }
    }

    pub async fn get_todo_by_title(
        Extension(db): Extension<Arc<Database>>,
        Path(title): Path<String>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
        let repository = TodosRepository::new(db);
        match repository.get_by_title(title.clone()).await {
            Ok(todo) => Ok((StatusCode::OK, Json(todo))),
            Err(_) => Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Todo with title: {} not found", title)
                })),
            )),
        }
    }

    pub async fn create_todo(
        Extension(db): Extension<Arc<Database>>,
        Json(body): Json<CreateTodo>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
        let repository = TodosRepository::new(db);
        let todo = Todo {
            id: None,
            title: body.title.clone(),
            content: Some(body.content.clone().unwrap_or("".to_string())),
            completed: Some(body.completed.unwrap_or(false)),
            created_at: Some(Local::now()),
            updated_at: None,
        };
        match repository.create(todo).await {
            Ok(todo) => {
                let json_response = serde_json::json!({
                    "status": "success",
                    "todo": todo.to_owned(),
                });
                Ok((StatusCode::CREATED, Json(json_response)))
            }
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "error",
                    "message": "Failed to create todo"
                })),
            )),
        }
    }



    pub async fn update_todo(
        Extension(db): Extension<Arc<Database>>,
        Path(id): Path<String>,
        Json(body): Json<UpdateTodo>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
        let repository = TodosRepository::new(db);

        match repository.get_by_id(id.clone()).await {
            Ok(mut todo) => {
                let datetime = Local::now();
                todo.title = body.title.clone().unwrap_or(todo.title);
                todo.content = body.content.or(todo.content);
                todo.completed = body.completed.or(todo.completed);
                todo.updated_at = Some(datetime);

                match repository.update(id.clone(), todo.clone()).await {
                    Ok(todo_response) => Ok((
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "status": "success",
                            "todo": todo_response
                        })),
                    )),
                    Err(_) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "status": "error",
                            "message": "Failed to update todo"
                        })),
                    )),
                }
            }
            Err(_) => Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Todo with ID: {} not found", id)
                })),
            )),
        }
    }

    pub async fn delete_todo(
        Extension(db): Extension<Arc<Database>>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
        let repository = TodosRepository::new(db);

        if repository.get_by_id(id.clone()).await.is_ok() {
            repository.delete(id.clone()).await.unwrap();
            let json_response = serde_json::json!({
                "status": "success",
                "message": "Todo deleted successfully"
            });
            Ok((StatusCode::NO_CONTENT, Json(json_response)))
        } else {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Todo with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}
