format:
    ruff format .
    ruff check --fix --select I .

run-app:
    uv run streamlit run app.py