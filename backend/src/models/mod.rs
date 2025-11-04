use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub question_number: i32,
    pub problem: String,
    pub done: bool,
    pub difficulty: String,
    pub tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionUpdate {
    pub done: Option<bool>,
    pub difficulty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsUpdate {
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub tag_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTag {
    pub tag_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInfo {
    pub name: String,
    pub display_name: String,
    pub total_questions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionInfo {
    pub id: String,
    pub display_name: String,
    pub list1: String,
    pub list2: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total: i32,
    pub completed: i32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub github_id: i64,
    pub username: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub github_id: i64,
    pub username: String,
    pub avatar_url: String,
    pub created_at: String,
}
