# Development Plan & TDD Strategy

This document outlines the phased, test-driven development plan for `md-splice`.

## Phase 1: The Locator

**Goal**: Implement the node selection logic in `src/locator.rs`.

**Test Cases**:
- **L1 (Simple Type)**: Select the first paragraph.
- **L2 (Type & Ordinal)**: Select the *second* list.
- **L3 (Content Contains)**: Select a heading containing a specific word.
- **L4 (Content Regex)**: Select a paragraph matching a regex.
- **L5 (Combined Selectors)**: Select the 2nd paragraph that contains "Note".
- **L6 (No Match)**: Verify `SpliceError::NodeNotFound`.
- **L7 (Ambiguity Warning)**: Verify `log::warn!` output for multiple matches.

## Phase 2: The Splicer

**Goal**: Implement the AST modification logic in `src/splicer.rs`.

**Test Cases**:
- **S1 (Replace)**: Replace a paragraph.
- **S2 (Insert Before/After)**: Insert a block relative to another.
- **S3 (Insert into Container)**: `prepend-child`/`append-child` into a `BlockQuote`.
- **S4 (Heading Heuristic)**: `prepend-child`/`append-child` into a heading section.
- **S5 (Invalid Insertion)**: Verify `SpliceError::InvalidChildInsertion`.

## Phase 3: CLI & Integration

**Goal**: Wire everything together and perform end-to-end testing.

**Test Cases**:
- **I1 (Help and Version)**: Test `--help` and `--version`.
- **I2 (File I/O)**: Test `--output` with `insta` snapshots.
- **I3 (In-Place Edit)**: Test in-place modification.
- **I4 (Content Sources)**: Test `--content` and `--content-file`.
- **I5 (Error Reporting)**: Test non-zero exit code on error.
- **I6 (Logging)**: Test log-based warning.

## Phase 4: List Item Selection

**Goal**: Implement the ability to select, replace, and insert individual list items.

### Sub-Phase 4.1: Locator Extension (LL)

**Test Cases**:
- **LL1 (Select by Type and Ordinal)**: Select the 3rd list item in a document containing multiple lists.
- **LL2 (Select by Content)**: Select a list item using `--select-contains`.
- **LL3 (Select by Regex)**: Select a list item using `--select-regex`.
- **LL4 (No Match)**: Verify `SpliceError::NodeNotFound` when a list item selector finds nothing.
- **LL5 (Ambiguity)**: Verify the ambiguity warning is triggered when a selector matches multiple list items.

### Sub-Phase 4.2: Splicer Extension (LS)

**Test Cases**:
- **LS1 (Replace Item)**: Replace a single list item with another single list item.
- **LS2 (Insert Before/After Item)**: Insert a new list item relative to an existing one.
- **LS3 (Insert into Item)**: Use `prepend-child`/`append-child` to add a nested list inside an existing list item.
- **LS4 (Replace One with Many)**: Replace a single list item with multiple new list items.

### Sub-Phase 4.3: Integration (LI)

**Test Cases**:
- **LI1 (End-to-End Replace)**: Use the CLI to replace a list item by its content. Create an `insta` snapshot.
- **LI2 (End-to-End Insert)**: Use the CLI to insert a new list item before another, selected by ordinal. Create an `insta` snapshot.
- **LI3 (End-to-End Error)**: Verify a non-zero exit code when trying to `prepend-child` into a list item with content that is not a valid block.
- **LI4 (End-to-End Nested Insert)**: Use the CLI to insert a nested list into an existing list item using `insert --position append-child`. Create an `insta` snapshot.
