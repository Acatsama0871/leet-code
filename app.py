from pathlib import Path

import duckdb
import streamlit as st

st.set_page_config(page_title="LeetCode Question Tracker", layout="wide")

st.title("LeetCode Question Tracker")

# Define the database path
db_path = Path("data/02_state/leetcode.duckdb")

# Check if database exists
if not db_path.exists():
    st.error(
        f"Database not found at {db_path}. Please run 'python load_data.py' first."
    )
    st.stop()

# Connect to DuckDB (not read-only to allow updates)
con = duckdb.connect(str(db_path))


# Load tags from database
def get_tags():
    """Fetch tags from database, returns list starting with empty string"""
    tags_df = con.execute("SELECT tag_name FROM tags ORDER BY tag_name").df()
    return [""] + tags_df["tag_name"].tolist()


TAG_OPTIONS = get_tags()

# Define tables and their display names
tables = {
    "NeetCode 150": "neetcode_150",
    "NeetCode Meta List": "neetcode_meta_list",
    "LeetCode Meta (3 months)": "leetcode_meta_3mo",
    "Advanced Algorithms": "adv_algo_questions",
    "Pinterest": "pinterest",
}

# Define intersections
intersections = {
    "Advanced Algorithms ∩ NeetCode 150": """
        SELECT DISTINCT a."Question Number", a.Problem
        FROM adv_algo_questions a
        INNER JOIN neetcode_150 n ON a."Question Number" = n."Question Number"
        ORDER BY a."Question Number"
    """,
    "LeetCode Meta (3mo) ∩ NeetCode Meta List": """
        SELECT DISTINCT l."Question Number", l.Problem
        FROM leetcode_meta_3mo l
        INNER JOIN neetcode_meta_list n ON l."Question Number" = n."Question Number"
        ORDER BY l."Question Number"
    """,
    "Advanced Algorithms ∩ LeetCode Meta (3mo)": """
        SELECT DISTINCT a."Question Number", a.Problem
        FROM adv_algo_questions a
        INNER JOIN leetcode_meta_3mo l ON a."Question Number" = l."Question Number"
        ORDER BY a."Question Number"
    """,
    "Advanced Algorithms ∩ NeetCode Meta List": """
        SELECT DISTINCT a."Question Number", a.Problem
        FROM adv_algo_questions a
        INNER JOIN neetcode_meta_list n ON a."Question Number" = n."Question Number"
        ORDER BY a."Question Number"
    """,
}

# Navigation in collapsible sidebar
with st.sidebar:
    st.markdown("### 📋 Navigation")

    # First level: Category selection
    category = st.selectbox("Category:", ["Raw", "Intersection", "Add Tags"])

    # Second level: Item selection based on category
    if category == "Raw":
        selected_item = st.selectbox("Select list:", list(tables.keys()))
    elif category == "Intersection":
        selected_item = st.selectbox("Select intersection:", list(intersections.keys()))
    else:  # Add Tags
        selected_item = None

# Main content area
if category == "Raw":
    # Display individual list data
    assert selected_item is not None, "No list selected"
    table_name = tables[selected_item]
    try:
        # Query data from DuckDB joining with question_status and aggregating tags
        df = con.execute(f"""
            SELECT
                t."Question Number",
                t.Problem,
                COALESCE(qs.done, false) as done,
                COALESCE(qs.difficulty, '') as difficulty,
                COALESCE(STRING_AGG(qt.tag_name, '; '), '') as tags
            FROM {table_name} t
            LEFT JOIN question_status qs ON t."Question Number" = qs.question_number
            LEFT JOIN question_tags qt ON t."Question Number" = qt.question_number
            GROUP BY t."Question Number", t.Problem, qs.done, qs.difficulty
            ORDER BY t."Question Number"
        """).df()

        st.header(selected_item)

        # Display metrics
        total_questions = len(df)
        done_questions = df["done"].sum()
        col1, col2, col3 = st.columns(3)
        with col1:
            st.metric("Total Questions", total_questions)
        with col2:
            st.metric("Completed", done_questions)
        with col3:
            progress_pct = (
                (done_questions / total_questions * 100) if total_questions > 0 else 0
            )
            st.metric("Progress", f"{progress_pct:.1f}%")

        # Reorder columns: done, difficulty, tags, question number, problem
        df = df[["done", "difficulty", "tags", "Question Number", "Problem"]]

        # Display editable dataframe (tags are now read-only)
        edited_df = st.data_editor(
            df,
            width="stretch",
            hide_index=True,
            column_config={
                "done": st.column_config.CheckboxColumn(
                    "Done",
                    help="Mark as completed",
                    default=False,
                    width=60,
                ),
                "difficulty": st.column_config.SelectboxColumn(
                    "Difficulty",
                    help="Question difficulty",
                    options=["", "Easy", "Medium", "Hard"],
                    width=100,
                ),
                "tags": st.column_config.TextColumn(
                    "Tags",
                    help="Current tags (edit below)",
                    disabled=True,
                    width="medium",
                ),
                "Question Number": st.column_config.NumberColumn(
                    "Question #",
                    help="LeetCode question number",
                    format="%d",
                    disabled=True,
                    width=80,
                ),
                "Problem": st.column_config.TextColumn(
                    "Problem Name",
                    help="Name of the LeetCode problem",
                    disabled=True,
                    width="large",
                ),
            },
            disabled=["Question Number", "Problem", "tags"],
            key=f"editor_{table_name}",
        )

        # Check if done status or difficulty was edited
        if not edited_df["done"].equals(df["done"]) or not edited_df[  # type: ignore[attr-defined]
            "difficulty"
        ].equals(df["difficulty"]):  # type: ignore[attr-defined]
            # Update done status and difficulty
            for idx in edited_df.index:
                question_num = int(edited_df.loc[idx, "Question Number"].item())  # type: ignore[union-attr]
                done_status = bool(edited_df.loc[idx, "done"])
                difficulty_val = (
                    str(edited_df.loc[idx, "difficulty"])
                    if edited_df.loc[idx, "difficulty"]
                    else ""
                )

                # Update done status and difficulty in question_status table
                con.execute(
                    """
                    INSERT INTO question_status (question_number, done, difficulty)
                    VALUES (?, ?, ?)
                    ON CONFLICT (question_number)
                    DO UPDATE SET done = ?, difficulty = ?
                    """,
                    [
                        question_num,
                        done_status,
                        difficulty_val,
                        done_status,
                        difficulty_val,
                    ],
                )
            st.success("✓ Progress saved!")
            st.rerun()

        # Tag editor section
        st.subheader("Edit Tags")
        with st.expander("Select a question to edit its tags", expanded=False):
            if question_options := {
                f"{int(row['Question Number'])}: {row['Problem']}": int(
                    row["Question Number"]
                )
                for _, row in df.iterrows()
            }:
                selected_question_str = st.selectbox(
                    "Select question:",
                    options=list(question_options.keys()),
                    key=f"question_selector_{table_name}",
                )
                selected_question_num = question_options[selected_question_str]

                # Get current tags for this question
                current_tags_df = con.execute(
                    "SELECT tag_name FROM question_tags WHERE question_number = ?",
                    [selected_question_num],
                ).df()
                current_tags = (
                    current_tags_df["tag_name"].tolist()
                    if len(current_tags_df) > 0
                    else []
                )

                # Get all available tags
                all_tags_df = con.execute(
                    "SELECT tag_name FROM tags ORDER BY tag_name"
                ).df()
                all_tags = (
                    all_tags_df["tag_name"].tolist() if len(all_tags_df) > 0 else []
                )

                # Multiselect for tags
                selected_tags = st.multiselect(
                    "Tags:",
                    options=all_tags,
                    default=current_tags,
                    key=f"tag_selector_{table_name}_{selected_question_num}",
                )

                # Save button
                if st.button("Save Tags", key=f"save_tags_{table_name}"):
                    # Delete existing tags for this question
                    con.execute(
                        "DELETE FROM question_tags WHERE question_number = ?",
                        [selected_question_num],
                    )

                    # Insert new tags
                    for tag in selected_tags:
                        con.execute(
                            "INSERT INTO question_tags (question_number, tag_name) VALUES (?, ?)",
                            [selected_question_num, tag],
                        )

                    st.success(f"✓ Tags updated for question {selected_question_num}!")
                    st.rerun()

    except Exception as e:
        st.error(f"Error loading table '{table_name}': {e}")
elif category == "Intersection":
    # Display intersection analysis
    assert selected_item is not None, "No intersection selected"
    st.header(selected_item)

    try:
        query = intersections[selected_item]
        query = intersections[selected_item]

        # Query intersection and join with question_status and aggregate tags
        df = con.execute(f"""
            WITH intersection AS ({query})
            SELECT
                i."Question Number",
                i.Problem,
                COALESCE(qs.done, false) as done,
                COALESCE(qs.difficulty, '') as difficulty,
                COALESCE(STRING_AGG(qt.tag_name, '; '), '') as tags
            FROM intersection i
            LEFT JOIN question_status qs ON i."Question Number" = qs.question_number
            LEFT JOIN question_tags qt ON i."Question Number" = qt.question_number
            GROUP BY i."Question Number", i.Problem, qs.done, qs.difficulty
            ORDER BY i."Question Number"
        """).df()

        if len(df) > 0:
            # Display metrics
            total_questions = len(df)
            done_questions = df["done"].sum()
            col1, col2, col3 = st.columns(3)
            with col1:
                st.metric("Common Questions", total_questions)
            with col2:
                st.metric("Completed", done_questions)
            with col3:
                progress_pct = (
                    (done_questions / total_questions * 100)
                    if total_questions > 0
                    else 0
                )
                st.metric("Progress", f"{progress_pct:.1f}%")

            # Reorder columns: done, difficulty, tags, question number, problem
            df = df[["done", "difficulty", "tags", "Question Number", "Problem"]]

            # Display editable dataframe (tags are now read-only)
            edited_df = st.data_editor(
                df,
                width="stretch",
                hide_index=True,
                column_config={
                    "done": st.column_config.CheckboxColumn(
                        "Done",
                        help="Mark as completed",
                        default=False,
                        width=60,
                    ),
                    "difficulty": st.column_config.SelectboxColumn(
                        "Difficulty",
                        help="Question difficulty",
                        options=["", "Easy", "Medium", "Hard"],
                        width=100,
                    ),
                    "tags": st.column_config.TextColumn(
                        "Tags",
                        help="Current tags (edit below)",
                        disabled=True,
                        width="medium",
                    ),
                    "Question Number": st.column_config.NumberColumn(
                        "Question #",
                        help="LeetCode question number",
                        format="%d",
                        disabled=True,
                        width=80,
                    ),
                    "Problem": st.column_config.TextColumn(
                        "Problem Name",
                        help="Name of the LeetCode problem",
                        disabled=True,
                        width="large",
                    ),
                },
                disabled=["Question Number", "Problem", "tags"],
                key=f"editor_intersection_{selected_item}",
            )

            # Check if done status or difficulty was edited
            if not edited_df["done"].equals(df["done"]) or not edited_df[  # type: ignore[attr-defined]
                "difficulty"
            ].equals(df["difficulty"]):  # type: ignore[attr-defined]
                # Update done status and difficulty
                for idx in edited_df.index:
                    question_num = int(edited_df.loc[idx, "Question Number"])  # type: ignore[union-attr]
                    done_status = bool(edited_df.loc[idx, "done"])
                    difficulty_val = (
                        str(edited_df.loc[idx, "difficulty"])
                        if edited_df.loc[idx, "difficulty"]
                        else ""
                    )

                    # Update done status and difficulty in question_status table
                    con.execute(
                        """
                        INSERT INTO question_status (question_number, done, difficulty)
                        VALUES (?, ?, ?)
                        ON CONFLICT (question_number)
                        DO UPDATE SET done = ?, difficulty = ?
                        """,
                        [
                            question_num,
                            done_status,
                            difficulty_val,
                            done_status,
                            difficulty_val,
                        ],
                    )
                st.success("✓ Progress saved!")
                st.rerun()

            # Tag editor section
            st.subheader("Edit Tags")
            with st.expander("Select a question to edit its tags", expanded=False):
                if question_options := {
                    f"{int(row['Question Number'])}: {row['Problem']}": int(
                        row["Question Number"]
                    )
                    for _, row in df.iterrows()
                }:
                    selected_question_str = st.selectbox(
                        "Select question:",
                        options=list(question_options.keys()),
                        key=f"question_selector_intersection_{selected_item}",
                    )
                    selected_question_num = question_options[selected_question_str]

                    # Get current tags for this question
                    current_tags_df = con.execute(
                        "SELECT tag_name FROM question_tags WHERE question_number = ?",
                        [selected_question_num],
                    ).df()
                    current_tags = (
                        current_tags_df["tag_name"].tolist()
                        if len(current_tags_df) > 0
                        else []
                    )

                    # Get all available tags
                    all_tags_df = con.execute(
                        "SELECT tag_name FROM tags ORDER BY tag_name"
                    ).df()
                    all_tags = (
                        all_tags_df["tag_name"].tolist() if len(all_tags_df) > 0 else []
                    )

                    # Multiselect for tags
                    selected_tags = st.multiselect(
                        "Tags:",
                        options=all_tags,
                        default=current_tags,
                        key=f"tag_selector_intersection_{selected_item}_{selected_question_num}",
                    )

                    # Save button
                    if st.button(
                        "Save Tags", key=f"save_tags_intersection_{selected_item}"
                    ):
                        # Delete existing tags for this question
                        con.execute(
                            "DELETE FROM question_tags WHERE question_number = ?",
                            [selected_question_num],
                        )

                        # Insert new tags
                        for tag in selected_tags:
                            con.execute(
                                "INSERT INTO question_tags (question_number, tag_name) VALUES (?, ?)",
                                [selected_question_num, tag],
                            )

                        st.success(
                            f"✓ Tags updated for question {selected_question_num}!"
                        )
                        st.rerun()

        else:
            st.info("No common questions found between these lists.")
    except Exception as e:
        st.error(f"Error computing intersection: {e}")

else:  # Add Tags
    # Tag management interface
    st.header("Tag Management")

    st.markdown("""
    Add custom tags to categorize your LeetCode problems. Tags will be available
    in the dropdown when marking questions across all lists.
    """)

    # Add new tag
    st.subheader("Add New Tag")
    col1, col2 = st.columns([3, 1])
    with col1:
        new_tag = st.text_input("Tag name:", key="new_tag_input")
    with col2:
        st.write("")  # Spacer
        st.write("")  # Spacer
        if st.button("Add Tag"):
            if new_tag and new_tag.strip():
                tag_name = new_tag.strip()
                try:
                    # Check if tag already exists
                    existing = con.execute(
                        "SELECT COUNT(*) FROM tags WHERE tag_name = ?", [tag_name]
                    ).fetchone()[0]  # type: ignore[index]

                    if existing > 0:
                        st.warning(f"Tag '{tag_name}' already exists!")
                    else:
                        con.execute(
                            "INSERT INTO tags (tag_name) VALUES (?)", [tag_name]
                        )
                        st.success(f"✓ Added tag: {tag_name}")
                        st.rerun()
                except Exception as e:
                    st.error(f"Error adding tag: {e}")
            else:
                st.warning("Please enter a tag name")

    # Display existing tags
    st.subheader("Existing Tags")
    tags_df = con.execute("SELECT tag_name FROM tags ORDER BY tag_name").df()

    if len(tags_df) > 0:
        # Show count
        st.metric("Total Tags", len(tags_df))

        # Display tags with delete buttons
        for idx, row in tags_df.iterrows():
            tag_name = row["tag_name"]
            col1, col2 = st.columns([4, 1])
            with col1:
                st.text(tag_name)
            with col2:
                if st.button("Delete", key=f"delete_{tag_name}_{idx}"):
                    try:
                        # First delete from question_tags (child table)
                        con.execute(
                            "DELETE FROM question_tags WHERE tag_name = ?", [tag_name]
                        )
                        # Then delete from tags (parent table)
                        con.execute("DELETE FROM tags WHERE tag_name = ?", [tag_name])
                        st.success(f"✓ Deleted tag: {tag_name}")
                        st.rerun()
                    except Exception as e:
                        st.error(f"Error deleting tag: {e}")
    else:
        st.info("No tags yet. Add some tags above to get started!")

# Close connection
con.close()
