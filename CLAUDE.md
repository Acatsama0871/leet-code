# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a LeetCode Question Tracker application built with Python, Streamlit, and DuckDB. It helps you manage and track your progress across multiple LeetCode question lists, with features for marking completion, setting difficulty levels, adding custom tags, and analyzing intersections between different lists.

## Project Structure

- `main.py`: CLI entry point using Typer framework (contains question_union command)
- `app.py`: Streamlit web interface for tracking and managing LeetCode questions
- `load_data.py`: Script to load CSV files from `data/01_raw/` into DuckDB and manage database schema
- `data/`: Contains LeetCode question lists
  - `01_raw/`: Raw CSV files with question lists (format: `Question Number,Problem`)
  - `02_state/`: Contains `leetcode.duckdb` - the DuckDB database with all question lists and progress tracking

## Technology Stack

- **CLI Framework**: Typer for command-line interface
- **Database**: DuckDB for data processing and state management
- **UI**: Streamlit for interactive web interface
- **Data Processing**: Pandas for DataFrame operations
- **Code Quality**: Ruff for linting/formatting, Pyright for type checking

## Features

### 1. Progress Tracking
- ✅ **Done Checkbox**: Mark questions as completed
- 📊 **Progress Metrics**: See total questions, completed count, and progress percentage
- 🔄 **Auto-refresh**: Metrics update automatically when you check/uncheck

### 2. Difficulty Levels
- 🎯 **Difficulty Selectbox**: Categorize questions as Easy, Medium, or Hard
- 💾 **Persistent Storage**: Difficulty settings saved to database

### 3. Custom Tagging System
- 🏷️ **Dynamic Tags**: Create custom tags for categorizing questions
- 🎨 **Multiple Tags**: Assign multiple tags to each question
- 🎛️ **Tag Management**: Add/delete tags via dedicated interface
- 🔗 **Tag Consistency**: Tags are shared across all lists

### 4. Multiple Views
- 📋 **Raw Lists**: View individual question lists
- 🔗 **Intersections**: See common questions between lists
  - Advanced Algorithms ∩ NeetCode 150
  - LeetCode Meta (3mo) ∩ NeetCode Meta List
  - Advanced Algorithms ∩ LeetCode Meta (3mo)
  - Advanced Algorithms ∩ NeetCode Meta List

### 5. Interactive UI
- ✏️ **Inline Editing**: Edit done status and difficulty directly in the table
- 🎚️ **Tag Editor**: Expandable section with multiselect for easy tag assignment
- 📏 **Optimized Layout**: Compact columns for done/question#, larger space for problem names

## Database Schema

### Core Tables

#### Question List Tables (5 tables)
- `neetcode_150` - NeetCode's 150 curated problems (150 questions)
- `neetcode_meta_list` - NeetCode's Meta-focused problems (52 questions)
- `leetcode_meta_3mo` - Meta interview questions (3-month frequency) (280 questions)
- `adv_algo_questions` - Advanced algorithm questions (59 questions)
- `pinterest` - Pinterest-specific interview questions (15 questions)

**Schema**:
```sql
CREATE TABLE <table_name> (
    "Question Number" INTEGER,
    "Problem" VARCHAR
)
```

#### Question Status Table
Tracks completion status and difficulty for all questions across all lists.

```sql
CREATE TABLE question_status (
    question_number INTEGER PRIMARY KEY,
    done BOOLEAN,
    difficulty VARCHAR  -- '', 'Easy', 'Medium', or 'Hard'
)
```

**Total Questions**: 380 unique questions across all lists

#### Tags Table
Stores custom tags that can be assigned to questions.

```sql
CREATE TABLE tags (
    tag_name VARCHAR PRIMARY KEY
)
```

#### Question Tags Junction Table
Many-to-many relationship between questions and tags.

```sql
CREATE TABLE question_tags (
    question_number INTEGER,
    tag_name VARCHAR,
    PRIMARY KEY (question_number, tag_name),
    FOREIGN KEY (question_number) REFERENCES question_status(question_number),
    FOREIGN KEY (tag_name) REFERENCES tags(tag_name)
)
```

### Data Relationships

```
question_status (1) ←→ (∞) question_tags (∞) ←→ (1) tags
         ↑
         |
    (FK via Question Number)
         |
    question lists (neetcode_150, etc.)
```

## Development Commands

### Dependency Management
This project uses `uv` for dependency management:
```bash
# Install dependencies
uv sync

# Add new dependency
uv add <package-name>
```

### Data Loading
Before running the app for the first time, or after schema changes, load CSV data into DuckDB:
```bash
# Load CSV files into DuckDB (required on first run)
uv run python load_data.py
```

This script will:
- Load all CSV files from `data/01_raw/` into DuckDB tables
- Create/update the database schema
- Preserve existing progress data (done status, difficulty, tags)
- Migrate tags from old schema if needed

### Running the Application
```bash
# Run CLI commands
python main.py question-union

# Or with uv
uv run python main.py question-union

# Run Streamlit web interface
uv run streamlit run app.py
# Opens at http://localhost:8501 or http://localhost:8502
```

### Code Quality
```bash
# Run ruff for linting and formatting
uv run ruff check .
uv run ruff format .

# Run pyright for type checking
uv run pyright
```

## Key Implementation Details

### Data Flow
1. **Raw CSV files** in `data/01_raw/` (problem names may contain commas like "Pow(x, n)")
2. **load_data.py** reads CSVs with `usecols=[0, 1]` to handle commas, loads into DuckDB
3. **DuckDB database** stored at `data/02_state/leetcode.duckdb`
4. **Streamlit app** (app.py) reads and writes to DuckDB (not read-only anymore)

### Query Pattern for Displaying Questions
```sql
SELECT
    t."Question Number",
    t.Problem,
    COALESCE(qs.done, false) as done,
    COALESCE(qs.difficulty, '') as difficulty,
    COALESCE(STRING_AGG(qt.tag_name, '; '), '') as tags
FROM <table_name> t
LEFT JOIN question_status qs ON t."Question Number" = qs.question_number
LEFT JOIN question_tags qt ON t."Question Number" = qt.question_number
GROUP BY t."Question Number", t.Problem, qs.done, qs.difficulty
ORDER BY t."Question Number"
```

**Key Points**:
- Uses `LEFT JOIN` so questions without status/tags still appear
- `STRING_AGG` concatenates multiple tags with "; " separator
- `COALESCE` provides default values for questions without status

### Saving Changes Pattern
When user edits done status or difficulty:

```python
# Check if data changed
if not edited_df["done"].equals(df["done"]) or not edited_df["difficulty"].equals(df["difficulty"]):
    # Update each changed row
    for idx in edited_df.index:
        question_num = int(edited_df.loc[idx, "Question Number"])
        done_status = bool(edited_df.loc[idx, "done"])
        difficulty_val = str(edited_df.loc[idx, "difficulty"]) if edited_df.loc[idx, "difficulty"] else ""

        # UPSERT pattern
        con.execute("""
            INSERT INTO question_status (question_number, done, difficulty)
            VALUES (?, ?, ?)
            ON CONFLICT (question_number)
            DO UPDATE SET done = ?, difficulty = ?
        """, [question_num, done_status, difficulty_val, done_status, difficulty_val])

    st.success("✓ Progress saved!")
    st.rerun()  # Refresh page to show updated metrics
```

### Tag Management Pattern
Tags use a separate expandable section with multiselect:

```python
# Get current tags for selected question
current_tags_df = con.execute(
    "SELECT tag_name FROM question_tags WHERE question_number = ?",
    [selected_question_num]
).df()
current_tags = current_tags_df["tag_name"].tolist()

# Multiselect widget
selected_tags = st.multiselect(
    "Tags:",
    options=all_tags,  # From tags table
    default=current_tags
)

# Save: delete old tags, insert new ones
if st.button("Save Tags"):
    con.execute("DELETE FROM question_tags WHERE question_number = ?", [selected_question_num])
    for tag in selected_tags:
        con.execute(
            "INSERT INTO question_tags (question_number, tag_name) VALUES (?, ?)",
            [selected_question_num, tag]
        )
```

### Column Configuration
Optimized column widths for better space utilization:

```python
column_config={
    "done": st.column_config.CheckboxColumn(
        "Done",
        width=60,  # Compact checkbox column
    ),
    "difficulty": st.column_config.SelectboxColumn(
        "Difficulty",
        options=["", "Easy", "Medium", "Hard"],
        width=100,
    ),
    "tags": st.column_config.TextColumn(
        "Tags",
        disabled=True,  # Read-only, edit via expander
        width="medium",
    ),
    "Question Number": st.column_config.NumberColumn(
        "Question #",
        disabled=True,
        width=80,  # Compact number column
    ),
    "Problem": st.column_config.TextColumn(
        "Problem Name",
        disabled=True,
        width="large",  # Takes remaining space
    ),
}
```

### Application Structure in app.py

1. **Sidebar Navigation** (lines 68-81)
   - Category selector: "Raw", "Intersection", "Add Tags"
   - Item selector based on category

2. **Raw Lists View** (lines 84-229)
   - Query data with joins
   - Display metrics
   - Show editable data_editor
   - Check for changes and save
   - Tag editor in expander

3. **Intersection View** (lines 230-379)
   - Same structure as Raw Lists
   - Uses CTE for intersection query

4. **Tag Management View** (lines 381-444)
   - Add new tags
   - Display existing tags
   - Delete tags (with cascade to question_tags)

## Data Sources

The repository manages multiple LeetCode question lists from CSV files in `data/01_raw/`:
- **neetcode_150.csv**: NeetCode's curated 150 problems (150 questions)
- **neetcode_meta_list.csv**: NeetCode's Meta-focused problems (52 questions)
- **leetcode_meta_3mo.csv**: Meta interview questions (3-month frequency) (280 questions)
- **adv_algo_questions.csv**: Advanced algorithm questions (59 questions)
- **pinterest.csv**: Pinterest-specific interview questions (15 questions)

**Total**: 380 unique questions across all lists

All CSV data is loaded into DuckDB tables for efficient querying and persistence.

## Important Notes

### Type Conversion for DuckDB
When saving data from Pandas DataFrames to DuckDB, convert numpy types to Python native types:

```python
question_num = int(edited_df.loc[idx, "Question Number"])  # numpy.int64 → int
done_status = bool(edited_df.loc[idx, "done"])  # numpy.bool_ → bool
difficulty_val = str(edited_df.loc[idx, "difficulty"])  # object → str
```

### Foreign Key Constraints
When dropping tables with foreign key relationships, drop child tables first:

```python
# Drop question_tags first (has FK to question_status)
con.execute("DROP TABLE IF EXISTS question_tags")
# Then drop question_status
con.execute("DROP TABLE IF EXISTS question_status")
```

### Tag Separator
Tags are stored in a junction table but displayed as a concatenated string:
- **Storage**: Multiple rows in `question_tags` table
- **Display**: Semicolon-separated string (e.g., "Array; Hash Table; Sliding Window")
- **Separator**: `"; "` (semicolon + space)

### Data Preservation
The `load_data.py` script preserves existing data when reloading:
- Checks for existing columns before reading
- Preserves `done` status
- Preserves `difficulty` values
- Preserves `tags` (old schema) and migrates to new schema
- Preserves `question_tags` relationships

## Recent Updates

### Latest Session Changes (2024)
1. ✅ Added difficulty selectbox column (Easy, Medium, Hard)
2. ✅ Further optimized column widths (Done: 60px, Question#: 80px, Difficulty: 100px)
3. ✅ Updated database schema with difficulty column
4. ✅ Modified both Raw and Intersection views to support difficulty

### Previous Major Changes
1. ✅ Removed hardcoded tag options, created dynamic tag system
2. ✅ Implemented many-to-many tag relationship with junction table
3. ✅ Added tag management interface (Add Tags category)
4. ✅ Implemented st.multiselect for tag assignment
5. ✅ Added auto-refresh on progress changes
6. ✅ Made tags column read-only with separate editor

## Troubleshooting

### Database Errors
If you encounter database schema errors:
```bash
# Backup your database first
cp data/02_state/leetcode.duckdb data/02_state/leetcode.duckdb.backup

# Reload data (this will preserve existing status/tags)
uv run python load_data.py
```

### Type Errors
If you see "Unable to transform python value" errors, ensure type conversion:
- Use `int()`, `bool()`, `str()` when passing DataFrame values to DuckDB

### Streamlit Cache Issues
If UI shows stale data:
- The app uses `st.rerun()` to refresh automatically
- If issues persist, restart the Streamlit server
