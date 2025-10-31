//! Defines custom error types for the application.

use thiserror::Error;

#[derive(Error, Debug)]
/// Error type returned when Markdown manipulation fails.
pub enum SpliceError {
    #[error("Selector did not match any nodes in the document")]
    NodeNotFound,

    #[error("Invalid operation: Cannot insert child content into a '{0}'. Use --position 'before' or 'after' to insert as a sibling.")]
    InvalidChildInsertion(String),

    #[error("Both --content and --content-file were provided. Please choose one.")]
    AmbiguousContentSource,

    #[error(
        "Neither --content nor --content-file were provided. Please specify the content to insert."
    )]
    NoContent,

    #[error("Invalid content for list item operation: content must be parsable as list items (e.g., '- item').")]
    InvalidListItemContent,

    #[error("Cannot read both source document and splice content from stdin.")]
    AmbiguousStdinSource,

    #[error("The --section flag can only be used when deleting a heading (h1-h6).")]
    InvalidSectionDelete,

    #[error("The --section flag can only be used when targeting a heading (h1-h6).")]
    SectionRequiresHeading,

    #[error("Cannot combine --after-* and --within-* selectors in the same query.")]
    ConflictingScopeModifiers,

    #[error("Range selectors are only supported for block-level selections.")]
    RangeRequiresBlock,

    #[error("Selector alias '{0}' was referenced before being defined.")]
    SelectorAliasNotDefined(String),

    #[error("Selector alias '{0}' has already been defined.")]
    SelectorAliasAlreadyDefined(String),

    #[error("Selector must specify exactly one of '{0}' or '{0}_ref'.")]
    AmbiguousSelectorSource(String),

    #[error("Nested selector must specify exactly one of '{0}' or '{0}_ref'.")]
    AmbiguousNestedSelectorSource(String),

    #[error("No frontmatter exists in the document.")]
    FrontmatterMissing,

    #[error("Frontmatter key '{0}' was not found.")]
    FrontmatterKeyNotFound(String),

    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterParse(String),

    #[error("Failed to serialize frontmatter: {0}")]
    FrontmatterSerialize(String),

    #[error("Failed to parse Markdown: {0}")]
    MarkdownParse(String),

    #[error("Failed to parse operations: {0}")]
    OperationParse(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("I/O error: {0}")]
    Io(String),
}
