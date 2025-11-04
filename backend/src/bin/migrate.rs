use anyhow::Result;
use duckdb::Connection;
use std::path::Path;

fn main() -> Result<()> {
    println!("🔄 Starting data migration...");

    let old_db_path = "./data/02_state/leetcode.duckdb";
    let new_db_path = "./data/leetcode.duckdb";

    // Check if old database exists
    if !Path::new(old_db_path).exists() {
        println!("❌ Old database not found at: {}", old_db_path);
        println!("ℹ️  If you're starting fresh, you can skip migration.");
        return Ok(());
    }

    // Check if new database already exists
    if Path::new(new_db_path).exists() {
        println!("⚠️  New database already exists at: {}", new_db_path);
        println!("⚠️  Skipping migration to avoid overwriting existing data.");
        println!("ℹ️  Delete the new database if you want to re-run migration.");
        return Ok(());
    }

    println!("📂 Opening old database: {}", old_db_path);
    let old_conn = Connection::open(old_db_path)?;

    println!("📂 Creating new database: {}", new_db_path);
    let new_conn = Connection::open(new_db_path)?;

    // Create tables in new database
    println!("🔨 Creating schema in new database...");
    create_schema(&new_conn)?;

    // Migrate question lists
    println!("📋 Migrating question lists...");
    migrate_question_lists(&old_conn, &new_conn)?;

    // Migrate question status
    println!("✅ Migrating question status (done, difficulty)...");
    migrate_question_status(&old_conn, &new_conn)?;

    // Migrate tags
    println!("🏷️  Migrating tags...");
    migrate_tags(&old_conn, &new_conn)?;

    println!("✨ Migration completed successfully!");
    println!("📊 New database ready at: {}", new_db_path);

    Ok(())
}

fn create_schema(conn: &Connection) -> Result<()> {
    // Create all tables
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS neetcode_150 (
            "Question Number" INTEGER,
            "Problem" VARCHAR
        );

        CREATE TABLE IF NOT EXISTS neetcode_meta_list (
            "Question Number" INTEGER,
            "Problem" VARCHAR
        );

        CREATE TABLE IF NOT EXISTS leetcode_meta_3mo (
            "Question Number" INTEGER,
            "Problem" VARCHAR
        );

        CREATE TABLE IF NOT EXISTS adv_algo_questions (
            "Question Number" INTEGER,
            "Problem" VARCHAR
        );

        CREATE TABLE IF NOT EXISTS pinterest (
            "Question Number" INTEGER,
            "Problem" VARCHAR
        );

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
        "#,
    )?;

    Ok(())
}

fn migrate_question_lists(old_conn: &Connection, new_conn: &Connection) -> Result<()> {
    let tables = vec![
        "neetcode_150",
        "neetcode_meta_list",
        "leetcode_meta_3mo",
        "adv_algo_questions",
        "pinterest",
    ];

    for table in tables {
        println!("  Copying table: {}", table);

        // Check if table exists in old database
        let table_exists: bool = old_conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_name = ?",
                [table],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !table_exists {
            println!("    ⚠️  Table {} not found in old database, skipping", table);
            continue;
        }

        // Get all rows from old table
        let mut stmt = old_conn.prepare(&format!(
            r#"SELECT "Question Number", "Problem" FROM {}"#,
            table
        ))?;

        let rows: Vec<(i32, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        // Insert into new table
        for (question_number, problem) in rows {
            new_conn.execute(
                &format!(
                    r#"INSERT INTO {} ("Question Number", "Problem") VALUES (?, ?)"#,
                    table
                ),
                [&question_number.to_string(), &problem],
            )?;
        }

        println!("    ✓ Copied {} rows", stmt.row_count());
    }

    Ok(())
}

fn migrate_question_status(old_conn: &Connection, new_conn: &Connection) -> Result<()> {
    // Check if question_status table exists
    let table_exists: bool = old_conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_name = 'question_status'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        println!("  ⚠️  question_status table not found, skipping");
        return Ok(());
    }

    let mut stmt = old_conn.prepare(
        "SELECT question_number, done, difficulty FROM question_status",
    )?;

    let rows: Vec<(i32, bool, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (question_number, done, difficulty) in rows {
        new_conn.execute(
            "INSERT INTO question_status (question_number, done, difficulty) VALUES (?, ?, ?)",
            [&question_number.to_string(), &done.to_string(), &difficulty],
        )?;
    }

    println!("  ✓ Migrated {} question statuses", stmt.row_count());

    Ok(())
}

fn migrate_tags(old_conn: &Connection, new_conn: &Connection) -> Result<()> {
    // Check if tags table exists
    let tags_exists: bool = old_conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_name = 'tags'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !tags_exists {
        println!("  ⚠️  tags table not found, skipping");
        return Ok(());
    }

    // Migrate tags
    let mut stmt = old_conn.prepare("SELECT tag_name FROM tags")?;
    let tags: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for tag in &tags {
        new_conn.execute("INSERT INTO tags (tag_name) VALUES (?)", [tag])?;
    }

    println!("  ✓ Migrated {} tags", tags.len());

    // Check if question_tags table exists
    let question_tags_exists: bool = old_conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_name = 'question_tags'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !question_tags_exists {
        println!("  ⚠️  question_tags table not found, skipping");
        return Ok(());
    }

    // Migrate question_tags
    let mut stmt = old_conn.prepare("SELECT question_number, tag_name FROM question_tags")?;
    let question_tags: Vec<(i32, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;

    for (question_number, tag_name) in &question_tags {
        new_conn.execute(
            "INSERT INTO question_tags (question_number, tag_name) VALUES (?, ?)",
            [&question_number.to_string(), tag_name],
        )?;
    }

    println!("  ✓ Migrated {} question-tag relationships", question_tags.len());

    Ok(())
}
