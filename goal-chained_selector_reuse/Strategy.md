# Strategy.md

## 1. Goal

The objective is to implement a selector reuse mechanism within `md-splice` transactions. This will allow users to define a selector with an `alias` in one operation and reference it in subsequent operations using `selector_ref`, reducing boilerplate in complex YAML/JSON transaction files. This strategy follows the "Selector Alias Handles" approach recommended in the preliminary study.

## 2. Overall Plan

The implementation will be broken down into the following milestones:

1. **Update Data Models:** Extend the Rust and Python data models for operations and selectors to support aliases and references.
2. **Enhance Error Handling:** Introduce new error types for alias-related failures.
3. **Implement Core Logic:** Modify the transaction execution engine in `md-splice-lib` to manage and resolve selector aliases.
4. **Update Python Bindings:** Ensure the new data model features are fully supported in the Python API, including object creation, validation, and serialization.
5. **Comprehensive Testing:** Add unit and integration tests for the new functionality in both the Rust library and the Python bindings.
6. **Update Documentation:** Revise the main `README.md` and the Python package's `README.md` to document the new feature with clear examples.

---

## Milestone 1: Update Data Models (Rust & Python)

The first step is to update the data structures that define transactions to accommodate aliases and references.

### 1.1. Rust Library (`md-splice-lib`)

**File to modify:** `md-splice-lib/src/transaction.rs`

1. **Extend `Selector` Struct:** Add an optional `alias` field to the `Selector` struct. This will be used to name a selector for later reuse.

    ```rust
    // In md-splice-lib/src/transaction.rs
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    pub struct Selector {
        #[serde(default)]
        pub alias: Option<String>, // New field
        // ... existing fields
    }
    ```

2. **Introduce Selector References:** Add `*_ref` fields to allow referencing an aliased selector.
    * In `InsertOperation`, `ReplaceOperation`, and `DeleteOperation`, add `selector_ref: Option<String>` and make the existing `selector: Selector` field optional. We will validate mutual exclusivity in the core logic.
    * In `Selector`, add `after_ref: Option<String>` and `within_ref: Option<String>`.
    * In `ReplaceOperation` and `DeleteOperation`, add `until_ref: Option<String>`.

    ```rust
    // In md-splice-lib/src/transaction.rs

    // Example for InsertOperation
    #[derive(Debug, Deserialize, PartialEq, Clone, Default)]
    pub struct InsertOperation {
        #[serde(default)]
        pub selector: Option<Selector>, // Make optional
        #[serde(default)]
        pub selector_ref: Option<String>, // New field
        // ... existing fields
    }

    // Example for Selector
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    pub struct Selector {
        #[serde(default)]
        pub alias: Option<String>,
        // ...
        #[serde(default)]
        pub after: Option<Box<Selector>>,
        #[serde(default)]
        pub after_ref: Option<String>, // New field
        #[serde(default)]
        pub within: Option<Box<Selector>>,
        #[serde(default)]
        pub within_ref: Option<String>, // New field
    }

    // Example for ReplaceOperation
    #[derive(Debug, Deserialize, PartialEq, Clone, Default)]
    pub struct ReplaceOperation {
        // ...
        #[serde(default)]
        pub until: Option<Selector>,
        #[serde(default)]
        pub until_ref: Option<String>, // New field
    }
    ```
    *Apply these changes to all relevant operation and selector structs.*

### 1.2. Python Bindings (`md-splice-py`)

**File to modify:** `md-splice-py/md_splice/types.py`

1. **Extend Python `Selector` Dataclass:** Add `alias`, `after_ref`, and `within_ref` fields. Add validation logic to `__post_init__` to ensure `after` and `after_ref` (and `within`/`within_ref`) are mutually exclusive.

    ```python
    # In md-splice-py/md_splice/types.py
    @dataclass(frozen=True, slots=True)
    class Selector:
        alias: str | None = None # New field
        # ... existing fields
        after: Selector | None = None
        after_ref: str | None = None # New field
        within: Selector | None = None
        within_ref: str | None = None # New field

        def __post_init__(self) -> None:
            # ... existing validation
            if self.after is not None and self.after_ref is not None:
                raise ValueError("Cannot specify both 'after' and 'after_ref'.")
            if self.within is not None and self.within_ref is not None:
                raise ValueError("Cannot specify both 'within' and 'within_ref'.")
    ```

2. **Extend Python Operation Dataclasses:** Add `selector_ref` to `InsertOperation`, `ReplaceOperation`, and `DeleteOperation`. Add `until_ref` to `ReplaceOperation` and `DeleteOperation`. Add validation logic.

    ```python
    # In md-splice-py/md_splice/types.py

    # Example for InsertOperation
    @dataclass(frozen=True, slots=True)
    class InsertOperation:
        selector: Selector | None = None
        selector_ref: str | None = None # New field
        # ... existing fields

        def __post_init__(self) -> None:
            if not ( (self.selector is None) ^ (self.selector_ref is None) ):
                raise ValueError("Must specify exactly one of 'selector' or 'selector_ref'.")

    # Example for ReplaceOperation
    @dataclass(frozen=True, slots=True)
    class ReplaceOperation:
        # ...
        until: Selector | None = None
        until_ref: str | None = None # New field

        def __post_init__(self) -> None:
            # ...
            if self.until is not None and self.until_ref is not None:
                raise ValueError("Cannot specify both 'until' and 'until_ref'.")
    ```
    *Apply these changes to all relevant operation dataclasses.*

## Milestone 2: Enhance Error Handling

We need new error types to provide clear feedback for invalid alias usage.

### 2.1. Rust Library (`md-splice-lib`)

**File to modify:** `md-splice-lib/src/error.rs`

1. **Add New `SpliceError` Variants:**
    ```rust
    // In md-splice-lib/src/error.rs
    #[derive(Error, Debug)]
    pub enum SpliceError {
        // ... existing variants
        #[error("Selector alias '{0}' is not defined.")]
        SelectorAliasNotDefined(String),

        #[error("Selector alias '{0}' is already defined.")]
        SelectorAliasAlreadyDefined(String),

        #[error("Operation must specify exactly one of 'selector' or 'selector_ref'.")]
        AmbiguousSelectorSource,

        #[error("Selector must specify exactly one of '{0}' or '{0}_ref'.")]
        AmbiguousNestedSelectorSource(String),
    }
    ```

### 2.2. Python Bindings (`md-splice-py`)

1. **File to modify:** `md-splice-py/md_splice/errors.py`
    * Add new exception classes inheriting from `MdSpliceError`:
        * `SelectorAliasNotDefinedError`
        * `SelectorAliasAlreadyDefinedError`
        * `AmbiguousSelectorSourceError`
        * `AmbiguousNestedSelectorSourceError`

2. **File to modify:** `md-splice-py/src/lib.rs`
    * Update the `map_splice_error_inner` function to map the new Rust `SpliceError` variants to their corresponding Python exceptions.

## Milestone 3: Implement Core Logic

This is the central part of the implementation, where we process the aliases during a transaction.

**File to modify:** `md-splice-lib/src/lib.rs`

1. **Modify `apply_operations_with_ambiguity`:**
    * At the beginning of the function, initialize a `HashMap<String, locator::Selector>` to store defined aliases.
    * The main loop over operations will now resolve selectors before execution.

2. **Create a Selector Resolution Helper Function:**
    * Implement a new private function, e.g., `resolve_selector_handle`, within `md-splice-lib/src/lib.rs`.
    * This function will take the selector-related fields from an operation (`selector`, `selector_ref`, etc.) and the alias map as input.
    * Its responsibilities:
        * Validate that exactly one of `selector` or `selector_ref` is provided, returning `SpliceError::AmbiguousSelectorSource` if not.
        * If `selector_ref` is used, look up the alias in the map. Return `SpliceError::SelectorAliasNotDefined` if not found.
        * If `selector` is used, recursively resolve its `after`, `within`, and `until` fields (which may also be references).
        * Return a fully resolved `locator::Selector`.

3. **Update the Main Operation Loop:**
    * Inside the `for operation in operations` loop in `apply_operations_with_ambiguity`:
        a. Call your new resolution helper to get a fully resolved `locator::Selector` for the current operation.
        b. Check if the resolved selector's source (`operation.selector`) defined an `alias`.
        c. If an alias was defined:
            i. Check if the alias already exists in the `HashMap`. If so, return `SpliceError::SelectorAliasAlreadyDefined`.
            ii. If not, clone the resolved `locator::Selector` and insert it into the `HashMap`.
        d. Pass the resolved `locator::Selector` to the existing logic that executes the operation (e.g., `apply_replace_operation`). The `build_locator_selector` function will need to be adapted or replaced by this new resolution logic.

## Milestone 4: Update Python Bindings

Update the Rust code for the Python bindings to correctly handle the new data model fields during conversions.

**File to modify:** `md-splice-py/src/lib.rs`

1. **Update `py_operation_to_rust` and `py_selector_to_transaction`:**
    * Modify these functions to read the new `*_ref` and `alias` fields from the Python dataclasses and populate the corresponding fields in the Rust `TxOperation` and `TxSelector` structs.

2. **Update `tx_operation_to_py` and `tx_selector_to_py`:**
    * These functions are used by `loads_operations`. Modify them to read the new fields from the Rust structs and populate the Python dataclasses correctly.

3. **Update `tx_operation_to_yaml_value` and `tx_selector_to_yaml_value`:**
    * These functions are used by `dumps_operations`. Modify them to serialize the new fields to the `YamlValue` representation.

## Milestone 5: Comprehensive Testing

Add tests to validate the new functionality and prevent regressions.

1. **Rust Library (`md-splice-lib`):**
    * In `md-splice-lib/src/transaction.rs` tests:
        * Add `serde_yaml` tests to verify that YAML with `alias` and `selector_ref` fields deserializes correctly.
        * Test failure cases, such as providing both `selector` and `selector_ref`.
    * In `md-splice-lib/src/lib.rs` tests:
        * Add a new test case for `apply_operations` that performs a multi-step transaction using a selector alias to modify the same heading section twice.
        * Add a test that expects a `SelectorAliasNotDefined` error.
        * Add a test that expects a `SelectorAliasAlreadyDefined` error.
        * Add a test that uses a nested reference (e.g., `within_ref`).

2. **Python Bindings (`md-splice-py`):**
    * Add tests for the `__post_init__` validation logic in the `types.py` dataclasses.
    * Add tests for `loads_operations` and `dumps_operations` to ensure they correctly handle YAML/JSON containing the new alias and reference fields.
    * Add end-to-end tests using `MarkdownDocument.apply` with `Selector(alias=...)` and `InsertOperation(selector_ref=...)`. Verify that the document is modified correctly.
    * Add tests that catch the new Python exceptions (`SelectorAliasNotDefinedError`, etc.) when invalid operations are applied.

## Milestone 6: Update Documentation

The final step is to document the new feature for users.

1. **File to modify:** `README.md` (root)
    * In the "Multi-operation transactions with `apply`" section, add a new subsection titled "Selector Reuse with Aliases".
    * Explain the purpose of `alias` and `selector_ref`.
    * Provide a clear, practical example of a transaction that modifies a heading section multiple times, showing the YAML *without* aliases and then the improved version *with* aliases to highlight the reduction in boilerplate. The example from the preliminary study is a perfect starting point.

2. **File to modify:** `md-splice-py/README.md`
    * In the "Quick usage" or a relevant section, add a small note and code snippet demonstrating how to use `alias` and `selector_ref` fields when constructing operation objects in Python. For example:
        ```python
        from md_splice import Selector, ReplaceOperation, InsertOperation

        ops = [
            ReplaceOperation(
                selector=Selector(
                    alias="changelog_h2",
                    select_type="h2",
                    select_contains="Changelog"
                ),
                content="## Changelog\n- First entry."
            ),
            InsertOperation(
                selector_ref="changelog_h2",
                position=InsertPosition.APPEND_CHILD,
                content="- Added selector reuse feature."
            )
        ]
        ```
