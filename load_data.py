"""
Script to load CSV files from data/01_raw into DuckDB database in data/02_state
"""

from pathlib import Path

import duckdb
import pandas as pd

# Define paths
raw_data_dir = Path("data/01_raw")
state_dir = Path("data/02_state")
db_path = state_dir / "leetcode.duckdb"

# Ensure state directory exists
state_dir.mkdir(parents=True, exist_ok=True)

# Define CSV files and their corresponding table names
csv_files = {
    "neetcode_150": "neetcode_150.csv",
    "neetcode_meta_list": "neetcode_meta_list.csv",
    "leetcode_meta_3mo": "leetcode_meta_3mo.csv",
    "adv_algo_questions": "adv_algo_questions.csv",
    "pinterest": "pinterest.csv",
}

# Connect to DuckDB
con = duckdb.connect(str(db_path))

print(f"Loading data into {db_path}")

# Load each CSV file into a DuckDB table
for table_name, filename in csv_files.items():
    file_path = raw_data_dir / filename

    if file_path.exists():
        print(f"Loading {filename} into table '{table_name}'...")

        # Read CSV with pandas (handling commas in problem names)
        df = pd.read_csv(
            file_path,
            usecols=[0, 1],
            names=["Question Number", "Problem"],
            header=0,  # type: ignore[call-overload]
        )

        # Create or replace table in DuckDB (without done/tag columns)
        con.execute(f"DROP TABLE IF EXISTS {table_name}")
        con.execute(f"CREATE TABLE {table_name} AS SELECT * FROM df")

        # Verify the data
        count = con.execute(f"SELECT COUNT(*) FROM {table_name}").fetchone()[0]  # type: ignore[index]
        print(f"  ✓ Loaded {count} questions into {table_name}")
    else:
        print(f"  ✗ File not found: {filename}")

# Create global question_status table
print("\nCreating global question_status table...")

# Collect all unique question numbers from all tables
all_questions = set()
for table_name in csv_files.keys():
    questions = con.execute(f'SELECT "Question Number" FROM {table_name}').df()
    all_questions.update(questions["Question Number"].tolist())

print(f"  Found {len(all_questions)} unique questions across all lists")

# Check if question_status table exists and preserve data
status_table_exists = (
    con.execute(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'question_status'"
    ).fetchone()[0]  # type: ignore[index]
    > 0
)

if status_table_exists:
    print("  Preserving existing status...")
    # Check if tag and difficulty columns exist in question_status
    columns = con.execute(
        "SELECT column_name FROM information_schema.columns WHERE table_name = 'question_status'"
    ).df()
    has_tag_column = "tag" in columns["column_name"].values
    has_difficulty_column = "difficulty" in columns["column_name"].values

    # Build query based on available columns
    select_cols = ["question_number", "done"]
    if has_difficulty_column:
        select_cols.append("difficulty")
    if has_tag_column:
        select_cols.append("tag")

    cols_str = ", ".join([f'"{col}"' for col in select_cols])
    query = f"SELECT {cols_str} FROM question_status"
    existing_status = con.execute(query).df()

    # Add missing columns
    if not has_tag_column:
        existing_status["tag"] = ""
    if not has_difficulty_column:
        existing_status["difficulty"] = ""
else:
    existing_status = pd.DataFrame(
        columns=["question_number", "done", "difficulty", "tag"]  # type: ignore[arg-type]
    )

# Create dataframe with all question numbers
status_df = pd.DataFrame({"question_number": sorted(all_questions)})

# Merge with existing status
status_df = status_df.merge(existing_status, on="question_number", how="left")
status_df["done"] = status_df["done"].fillna(False).astype(bool)
status_df["difficulty"] = status_df["difficulty"].fillna("")
status_df["tag"] = status_df["tag"].fillna("")

# Preserve question_tags data before dropping
question_tags_exists = (
    con.execute(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'question_tags'"
    ).fetchone()[0]  # type: ignore[index]
    > 0
)

if question_tags_exists:
    print("  Preserving existing question tags...")
    existing_question_tags = con.execute(
        'SELECT "question_number", "tag_name" FROM question_tags'
    ).df()
else:
    existing_question_tags = pd.DataFrame(columns=["question_number", "tag_name"])  # type: ignore[arg-type]

# Drop question_tags first if it exists (because of foreign key constraint)
con.execute("DROP TABLE IF EXISTS question_tags")

# Create or replace question_status table with primary key (without tag column)
con.execute("DROP TABLE IF EXISTS question_status")
con.execute("""
    CREATE TABLE question_status (
        question_number INTEGER PRIMARY KEY,
        done BOOLEAN,
        difficulty VARCHAR
    )
""")
# Insert question_number, done, and difficulty columns
con.execute(
    "INSERT INTO question_status SELECT question_number, done, difficulty FROM status_df"
)

print(f"  ✓ Created question_status table with {len(status_df)} questions")

# Create tags table for custom tags
print("\nCreating tags table...")
tags_table_exists = (
    con.execute(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'tags'"
    ).fetchone()[0]  # type: ignore[index]
    > 0
)

if not tags_table_exists:
    con.execute("""
        CREATE TABLE tags (
            tag_name VARCHAR PRIMARY KEY
        )
    """)
    print("  ✓ Created tags table")
else:
    print("  ✓ Tags table already exists")

# Create question_tags junction table for many-to-many relationship
print("\nCreating question_tags junction table...")
con.execute("""
    CREATE TABLE question_tags (
        question_number INTEGER,
        tag_name VARCHAR,
        PRIMARY KEY (question_number, tag_name),
        FOREIGN KEY (question_number) REFERENCES question_status(question_number),
        FOREIGN KEY (tag_name) REFERENCES tags(tag_name)
    )
""")
print("  ✓ Created question_tags junction table")

# Migrate existing tags from status_df to question_tags table
print("\nMigrating existing tags...")
migrated_count = 0
for _, row in status_df.iterrows():
    if row["tag"] and str(row["tag"]).strip():  # type: ignore[arg-type]
        question_num = int(row["question_number"])
        tag_value = str(row["tag"]).strip()
        try:
            # Insert tag into tags table if it doesn't exist
            con.execute("INSERT OR IGNORE INTO tags (tag_name) VALUES (?)", [tag_value])
            # Insert into question_tags junction table
            con.execute(
                "INSERT OR IGNORE INTO question_tags (question_number, tag_name) VALUES (?, ?)",
                [question_num, tag_value],
            )
            migrated_count += 1
        except Exception as e:
            print(
                f"  Warning: Could not migrate tag '{tag_value}' for question {question_num}: {e}"
            )

if migrated_count > 0:
    print(f"  ✓ Migrated {migrated_count} existing tags")
else:
    print("  ✓ No existing tags to migrate")

# Restore preserved question_tags data
print("\nRestoring preserved question tags...")
restored_count = 0
for _, row in existing_question_tags.iterrows():
    question_num = int(row["question_number"])
    tag_name = str(row["tag_name"])
    try:
        con.execute(
            "INSERT OR IGNORE INTO question_tags (question_number, tag_name) VALUES (?, ?)",
            [question_num, tag_name],
        )
        restored_count += 1
    except Exception as e:
        print(
            f"  Warning: Could not restore tag '{tag_name}' for question {question_num}: {e}"
        )

if restored_count > 0:
    print(f"  ✓ Restored {restored_count} question tags")
else:
    print("  ✓ No question tags to restore")

# Show summary
print("\nDatabase summary:")
tables = con.execute("SHOW TABLES").fetchall()
for table in tables:
    table_name = table[0]
    count = con.execute(f"SELECT COUNT(*) FROM {table_name}").fetchone()[0]  # type: ignore[index]
    print(f"  - {table_name}: {count} questions")

# Close connection
con.close()
print(f"\n✓ Data successfully loaded into {db_path}")
