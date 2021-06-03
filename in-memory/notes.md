# Cannot do this

```rust
#[get("/tasks")]
async fn insert(
    app_data: web::Data<AppData>,
    input: web::Json<InsertTask>,
) -> Result<HttpResponse, AppError> {
    if input.non_empty_title.trim().is_empty() {
        Err(AppError::EmptyTitle)
    } else {
        let id = app_data.id_tracker;
        let new_task = Task {
            id,
            title: input.non_empty_title,
            details: input.details,
        };

        // cannot borrow Arc as mutable
        app_data.task_list.push(new_task.clone());
        app_data.id_tracker += 1;

        let response = HttpResponse::Ok().json(new_task);
        Ok(response)
    }
}
```
