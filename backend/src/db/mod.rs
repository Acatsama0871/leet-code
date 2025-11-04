use crate::models::*;
use anyhow::Result;
use duckdb::Connection;
use std::sync::{Arc, Mutex};

pub type DbConnection = Arc<Mutex<Connection>>;

pub fn init_database(path: &str) -> Result<DbConnection> {
    let conn = Connection::open(path)?;

    // Create tables if they don't exist
    init_schema(&conn)?;

    Ok(Arc::new(Mutex::new(conn)))
}

fn init_schema(conn: &Connection) -> Result<()> {
    // Create question lists tables (these should already exist from CSV load)
    // We'll just ensure the question_status, tags, and question_tags tables exist

    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS question_status (
            question_number INTEGER PRIMARY KEY,
            done BOOLEAN DEFAULT FALSE,
            difficulty VARCHAR DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS tags (
            tag_name VARCHAR PRIMARY KEY
        );

        CREATE TABLE IF NOT EXISTS question_tags (
            question_number INTEGER,
            tag_name VARCHAR,
            PRIMARY KEY (question_number, tag_name),
            FOREIGN KEY (question_number) REFERENCES question_status(question_number),
            FOREIGN KEY (tag_name) REFERENCES tags(tag_name) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS user_sessions (
            session_id VARCHAR PRIMARY KEY,
            github_id BIGINT NOT NULL,
            username VARCHAR NOT NULL,
            avatar_url VARCHAR NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    "#)?;

    Ok(())
}

// Get all available lists
pub fn get_lists(conn: &DbConnection) -> Result<Vec<ListInfo>> {
    let conn = conn.lock().unwrap();

    let lists = vec![
        ListInfo {
            name: "neetcode_150".to_string(),
            display_name: "NeetCode 150".to_string(),
            total_questions: 150,
        },
        ListInfo {
            name: "neetcode_meta_list".to_string(),
            display_name: "NeetCode Meta List".to_string(),
            total_questions: 52,
        },
        ListInfo {
            name: "leetcode_meta_3mo".to_string(),
            display_name: "LeetCode Meta (3mo)".to_string(),
            total_questions: 280,
        },
        ListInfo {
            name: "adv_algo_questions".to_string(),
            display_name: "Advanced Algorithms".to_string(),
            total_questions: 59,
        },
        ListInfo {
            name: "pinterest".to_string(),
            display_name: "Pinterest".to_string(),
            total_questions: 15,
        },
    ];

    Ok(lists)
}

// Get questions for a specific list
pub fn get_list_questions(conn: &DbConnection, list_name: &str) -> Result<Vec<Question>> {
    let conn = conn.lock().unwrap();

    let query = format!(
        r#"
        SELECT
            t."Question Number" as question_number,
            t.Problem as problem,
            COALESCE(qs.done, false) as done,
            COALESCE(qs.difficulty, '') as difficulty,
            COALESCE(STRING_AGG(qt.tag_name, '; '), '') as tags
        FROM {} t
        LEFT JOIN question_status qs ON t."Question Number" = qs.question_number
        LEFT JOIN question_tags qt ON t."Question Number" = qt.question_number
        GROUP BY t."Question Number", t.Problem, qs.done, qs.difficulty
        ORDER BY t."Question Number"
        "#,
        list_name
    );

    let mut stmt = conn.prepare(&query)?;
    let questions = stmt.query_map([], |row| {
        Ok(Question {
            question_number: row.get(0)?,
            problem: row.get(1)?,
            done: row.get(2)?,
            difficulty: row.get(3)?,
            tags: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(questions)
}

// Get available intersections
pub fn get_intersections(conn: &DbConnection) -> Result<Vec<IntersectionInfo>> {
    let _ = conn;

    let intersections = vec![
        IntersectionInfo {
            id: "adv_algo_neetcode_150".to_string(),
            display_name: "Advanced Algorithms ∩ NeetCode 150".to_string(),
            list1: "adv_algo_questions".to_string(),
            list2: "neetcode_150".to_string(),
        },
        IntersectionInfo {
            id: "leetcode_meta_3mo_neetcode_meta".to_string(),
            display_name: "LeetCode Meta (3mo) ∩ NeetCode Meta List".to_string(),
            list1: "leetcode_meta_3mo".to_string(),
            list2: "neetcode_meta_list".to_string(),
        },
        IntersectionInfo {
            id: "adv_algo_leetcode_meta_3mo".to_string(),
            display_name: "Advanced Algorithms ∩ LeetCode Meta (3mo)".to_string(),
            list1: "adv_algo_questions".to_string(),
            list2: "leetcode_meta_3mo".to_string(),
        },
        IntersectionInfo {
            id: "adv_algo_neetcode_meta".to_string(),
            display_name: "Advanced Algorithms ∩ NeetCode Meta List".to_string(),
            list1: "adv_algo_questions".to_string(),
            list2: "neetcode_meta_list".to_string(),
        },
    ];

    Ok(intersections)
}

// Get intersection questions
pub fn get_intersection_questions(
    conn: &DbConnection,
    list1: &str,
    list2: &str,
) -> Result<Vec<Question>> {
    let conn = conn.lock().unwrap();

    let query = format!(
        r#"
        WITH intersection AS (
            SELECT "Question Number", Problem
            FROM {}
            INTERSECT
            SELECT "Question Number", Problem
            FROM {}
        )
        SELECT
            i."Question Number" as question_number,
            i.Problem as problem,
            COALESCE(qs.done, false) as done,
            COALESCE(qs.difficulty, '') as difficulty,
            COALESCE(STRING_AGG(qt.tag_name, '; '), '') as tags
        FROM intersection i
        LEFT JOIN question_status qs ON i."Question Number" = qs.question_number
        LEFT JOIN question_tags qt ON i."Question Number" = qt.question_number
        GROUP BY i."Question Number", i.Problem, qs.done, qs.difficulty
        ORDER BY i."Question Number"
        "#,
        list1, list2
    );

    let mut stmt = conn.prepare(&query)?;
    let questions = stmt.query_map([], |row| {
        Ok(Question {
            question_number: row.get(0)?,
            problem: row.get(1)?,
            done: row.get(2)?,
            difficulty: row.get(3)?,
            tags: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(questions)
}

// Update question status
pub fn update_question(
    conn: &DbConnection,
    question_number: i32,
    update: &QuestionUpdate,
) -> Result<()> {
    let conn = conn.lock().unwrap();

    // First check if the question exists
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM question_status WHERE question_number = ?",
        [question_number],
        |row| row.get(0),
    ).unwrap_or(false);

    if exists {
        // Update existing record
        if let Some(done) = update.done {
            conn.execute(
                "UPDATE question_status SET done = ? WHERE question_number = ?",
                [&done.to_string(), &question_number.to_string()],
            )?;
        }
        if let Some(ref difficulty) = update.difficulty {
            conn.execute(
                "UPDATE question_status SET difficulty = ? WHERE question_number = ?",
                [difficulty, &question_number.to_string()],
            )?;
        }
    } else {
        // Insert new record
        let done = update.done.unwrap_or(false);
        let difficulty = update.difficulty.clone().unwrap_or_default();

        conn.execute(
            "INSERT INTO question_status (question_number, done, difficulty) VALUES (?, ?, ?)",
            [&question_number.to_string(), &done.to_string(), &difficulty],
        )?;
    }

    Ok(())
}

// Get all tags
pub fn get_tags(conn: &DbConnection) -> Result<Vec<Tag>> {
    let conn = conn.lock().unwrap();

    let mut stmt = conn.prepare("SELECT tag_name FROM tags ORDER BY tag_name")?;
    let tags = stmt.query_map([], |row| {
        Ok(Tag {
            tag_name: row.get(0)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(tags)
}

// Create a new tag
pub fn create_tag(conn: &DbConnection, tag_name: &str) -> Result<()> {
    let conn = conn.lock().unwrap();

    conn.execute(
        "INSERT INTO tags (tag_name) VALUES (?)",
        [tag_name],
    )?;

    Ok(())
}

// Delete a tag
pub fn delete_tag(conn: &DbConnection, tag_name: &str) -> Result<()> {
    let conn = conn.lock().unwrap();

    conn.execute("DELETE FROM tags WHERE tag_name = ?", [tag_name])?;

    Ok(())
}

// Get tags for a question
pub fn get_question_tags(conn: &DbConnection, question_number: i32) -> Result<Vec<String>> {
    let conn = conn.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT tag_name FROM question_tags WHERE question_number = ? ORDER BY tag_name"
    )?;
    let tags = stmt.query_map([question_number], |row| {
        row.get(0)
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(tags)
}

// Update question tags
pub fn update_question_tags(
    conn: &DbConnection,
    question_number: i32,
    tags: &[String],
) -> Result<()> {
    let conn = conn.lock().unwrap();

    // Delete existing tags
    conn.execute(
        "DELETE FROM question_tags WHERE question_number = ?",
        [question_number],
    )?;

    // Insert new tags
    for tag in tags {
        conn.execute(
            "INSERT INTO question_tags (question_number, tag_name) VALUES (?, ?)",
            [&question_number.to_string(), tag],
        )?;
    }

    Ok(())
}

// Get metrics for a list
pub fn get_metrics(conn: &DbConnection, list_name: &str) -> Result<Metrics> {
    let conn = conn.lock().unwrap();

    let query = format!(
        r#"
        SELECT
            COUNT(*) as total,
            SUM(CASE WHEN qs.done THEN 1 ELSE 0 END) as completed
        FROM {} t
        LEFT JOIN question_status qs ON t."Question Number" = qs.question_number
        "#,
        list_name
    );

    let (total, completed): (i32, Option<i32>) = conn.query_row(&query, [], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let completed = completed.unwrap_or(0);
    let percentage = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(Metrics {
        total,
        completed,
        percentage,
    })
}

// Session management
pub fn create_session(
    conn: &DbConnection,
    session_id: &str,
    github_id: i64,
    username: &str,
    avatar_url: &str,
) -> Result<()> {
    let conn = conn.lock().unwrap();

    conn.execute(
        "INSERT INTO user_sessions (session_id, github_id, username, avatar_url) VALUES (?, ?, ?, ?)",
        [&session_id.to_string(), &github_id.to_string(), username, avatar_url],
    )?;

    Ok(())
}

pub fn get_session(conn: &DbConnection, session_id: &str) -> Result<Option<Session>> {
    let conn = conn.lock().unwrap();

    let result = conn.query_row(
        "SELECT session_id, github_id, username, avatar_url, created_at FROM user_sessions WHERE session_id = ?",
        [session_id],
        |row| {
            Ok(Session {
                session_id: row.get(0)?,
                github_id: row.get(1)?,
                username: row.get(2)?,
                avatar_url: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    );

    match result {
        Ok(session) => Ok(Some(session)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_session(conn: &DbConnection, session_id: &str) -> Result<()> {
    let conn = conn.lock().unwrap();

    conn.execute("DELETE FROM user_sessions WHERE session_id = ?", [session_id])?;

    Ok(())
}
