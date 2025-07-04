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

