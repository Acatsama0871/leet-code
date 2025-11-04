# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Python CLI application for managing and analyzing LeetCode problem lists. It processes CSV files containing LeetCode questions from various sources (NeetCode, company-specific lists, algorithm topics) and provides utilities to work with them.

## Project Structure

- `main.py`: CLI entry point using Typer framework
- `data/`: Contains LeetCode question lists in CSV format
  - CSV format: `Question Number,Problem` (e.g., "217,Contains Duplicate")
  - Various curated lists: `neetcode_150.csv`, `leetcode_meta_3mo.csv`, `adv_algo_questions.csv`, `pintrest.csv`, etc.
  - `01_raw/`: Raw data directory (currently empty)
  - `02_state/`: State data directory (currently empty)

## Technology Stack

- **CLI Framework**: Typer for command-line interface
- **Database**: DuckDB for data processing
- **UI**: Rich for terminal output formatting, Streamlit for web interface
- **Code Quality**: Ruff for linting/formatting

## Development Commands

### Dependency Management
This project uses `uv` for dependency management:
```bash
# Install dependencies
uv sync

# Add new dependency
uv add <package-name>
```

### Running the Application
```bash
# Run CLI commands
python main.py question-union

# Or with uv
uv run python main.py question-union
```

### Code Quality
```bash
# Run ruff for linting and formatting
uv run ruff check .
uv run ruff format .
```

## Key Implementation Details

- The CLI is built with Typer, defining commands as decorated functions
- CSV files in `data/` contain LeetCode questions with consistent schema: Question Number and Problem name
- DuckDB is available for SQL-based analysis of question lists
- Rich library should be used for formatted terminal output
- Streamlit can be used for building interactive web interfaces for the data

## Data Sources

The repository manages multiple LeetCode question lists:
- **neetcode_150.csv**: NeetCode's curated 150 problems
- **neetcode_meta_list.csv**: NeetCode's Meta-focused problems
- **leetcode_meta_3mo.csv**: Meta interview questions (3-month frequency)
- **adv_algo_questions.csv**: Advanced algorithm questions
- **pintrest.csv**: Pinterest-specific interview questions
