use crate::auth::AppState;
use crate::db;
use crate::models::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

// Get all lists
pub async fn get_lists(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ListInfo>>, StatusCode> {
    let lists = db::get_lists(&state.db)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(lists))
}

// Get questions for a specific list
pub async fn get_list_questions(
    State(state): State<Arc<AppState>>,
    Path(list_name): Path<String>,
) -> Result<Json<Vec<Question>>, StatusCode> {
    let questions = db::get_list_questions(&state.db, &list_name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(questions))
}

// Get all intersections
pub async fn get_intersections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<IntersectionInfo>>, StatusCode> {
    let intersections = db::get_intersections(&state.db)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(intersections))
}

// Get intersection questions
pub async fn get_intersection_questions(
    State(state): State<Arc<AppState>>,
    Path(intersection_id): Path<String>,
) -> Result<Json<Vec<Question>>, StatusCode> {
    // Parse intersection_id to get the two lists
    let intersections = db::get_intersections(&state.db)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let intersection = intersections
        .iter()
        .find(|i| i.id == intersection_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let questions = db::get_intersection_questions(&state.db, &intersection.list1, &intersection.list2)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(questions))
}

// Update question
pub async fn update_question(
    State(state): State<Arc<AppState>>,
    Path(question_number): Path<i32>,
    Json(update): Json<QuestionUpdate>,
) -> Result<StatusCode, StatusCode> {
    db::update_question(&state.db, question_number, &update)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// Get all tags
pub async fn get_tags(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Tag>>, StatusCode> {
    let tags = db::get_tags(&state.db)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tags))
}

// Create tag
pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    Json(tag): Json<CreateTag>,
) -> Result<StatusCode, StatusCode> {
    db::create_tag(&state.db, &tag.tag_name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

// Delete tag
pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    Path(tag_name): Path<String>,
) -> Result<StatusCode, StatusCode> {
    db::delete_tag(&state.db, &tag_name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// Get question tags
pub async fn get_question_tags(
    State(state): State<Arc<AppState>>,
    Path(question_number): Path<i32>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let tags = db::get_question_tags(&state.db, question_number)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tags))
}

// Update question tags
pub async fn update_question_tags(
    State(state): State<Arc<AppState>>,
    Path(question_number): Path<i32>,
    Json(update): Json<TagsUpdate>,
) -> Result<StatusCode, StatusCode> {
    db::update_question_tags(&state.db, question_number, &update.tags)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// Get metrics for a list
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
    Path(list_name): Path<String>,
) -> Result<Json<Metrics>, StatusCode> {
    let metrics = db::get_metrics(&state.db, &list_name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metrics))
}
