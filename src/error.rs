//! Defines custom error types for the application.

use thiserror::Error;

#[derive(Error, Debug)]
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
}
