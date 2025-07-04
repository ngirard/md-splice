### Directory contents in a tree‐like format
.
├── src
│   ├── ast
│   │   └── mod.rs
│   ├── parser
│   │   ├── blocks
│   │   │   ├── tests
│   │   │   │   ├── blockquote.rs
│   │   │   │   ├── code_block.rs
│   │   │   │   ├── custom_parser.rs
│   │   │   │   ├── footnote_definition.rs
│   │   │   │   ├── heading.rs
│   │   │   │   ├── html_block.rs
│   │   │   │   ├── link_definition.rs
│   │   │   │   ├── list.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── paragraph.rs
│   │   │   │   ├── table.rs
│   │   │   │   └── thematic_break.rs
│   │   │   ├── blockquote.rs
│   │   │   ├── code_block.rs
│   │   │   ├── footnote_definition.rs
│   │   │   ├── heading.rs
│   │   │   ├── html_block.rs
│   │   │   ├── link_definition.rs
│   │   │   ├── list.rs
│   │   │   ├── mod.rs
│   │   │   ├── paragraph.rs
│   │   │   ├── table.rs
│   │   │   └── thematic_break.rs
│   │   ├── inline
│   │   │   ├── tests
│   │   │   │   ├── autolink.rs
│   │   │   │   ├── code_span.rs
│   │   │   │   ├── emphasis.rs
│   │   │   │   ├── footnote_reference.rs
│   │   │   │   ├── hard_newline.rs
│   │   │   │   ├── html_entity.rs
│   │   │   │   ├── image.rs
│   │   │   │   ├── inline_link.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── reference_link.rs
│   │   │   │   └── strikethrough.rs
│   │   │   ├── autolink.rs
│   │   │   ├── code_span.rs
│   │   │   ├── emphasis.rs
│   │   │   ├── footnote_reference.rs
│   │   │   ├── hard_newline.rs
│   │   │   ├── html_entity.rs
│   │   │   ├── image.rs
│   │   │   ├── inline_link.rs
│   │   │   ├── mod.rs
│   │   │   ├── reference_link.rs
│   │   │   ├── strikethrough.rs
│   │   │   └── text.rs
│   │   ├── config.rs
│   │   ├── link_util.rs
│   │   ├── mod.rs
│   │   └── util.rs
│   ├── printer
│   │   ├── tests
│   │   │   ├── list.rs
│   │   │   └── mod.rs
│   │   ├── blockquote.rs
│   │   ├── block.rs
│   │   ├── config.rs
│   │   ├── heading.rs
│   │   ├── inline.rs
│   │   ├── list.rs
│   │   ├── mod.rs
│   │   └── table.rs
│   └── lib.rs
├── Cargo.toml
└── README.md

### Contents of the non-binary files of the directory

`Cargo.toml`:

````
[package]
authors = ["Evgenii Lepikhin <johnlepikhin@gmail.com>"]
name = "markdown-ppp"
version = "2.0.1"
edition = "2021"
license = "MIT"
description = "Feature-rich Markdown Parsing and Pretty-Printing library"
repository = "https://github.com/johnlepikhin/markdown-ppp"
categories = ["parsing", "text-processing"]
keywords = ["markdown", "parser", "pretty-print", "format"]
documentation = "https://docs.rs/markdown-ppp"
readme = "README.md"

[dependencies]
entities = { version = "1.0.1", optional = true }
nom = { version = "8.0.0", default-features = false, features = ["alloc"], optional = true }
pretty = { version = "0.12.4", optional = true }
serde = { version = "1.0.219", features = ["serde_derive"], optional = true }
unicode_categories = { version = "0.1.1", optional = true }

[dev-dependencies]
rstest = "0.25"

[features]
default = ["parser", "printer", "html-printer"]
parser = ["entities", "nom", "unicode_categories"]
ast-serde = ["serde"]
printer = ["pretty"]
html-printer = ["pretty"]

````

`README.md`:

````
[![crates.io](https://img.shields.io/crates/v/markdown-ppp.svg)](https://crates.io/crates/markdown-ppp)
[![docs.rs](https://docs.rs/markdown-ppp/badge.svg)](https://docs.rs/markdown-ppp)
[![CI](https://github.com/johnlepikhin/markdown-ppp/actions/workflows/ci.yml/badge.svg)](https://github.com/johnlepikhin/markdown-ppp/actions)
[![License: MIT](https://img.shields.io/crates/l/markdown-ppp.svg)](https://github.com/johnlepikhin/markdown-ppp/blob/main/LICENSE)

# markdown-ppp

**markdown-ppp** is a feature-rich, flexible, and lightweight Rust library for parsing and processing Markdown documents.

It provides a clean, well-structured Abstract Syntax Tree (AST) for parsed documents, making it suitable for pretty-printing, analyzing, transforming, or rendering Markdown.

---

## ✨ Features

- **Markdown Parsing** — Full Markdown parsing support with strict AST structure.
- **Pretty-printing and processing** — Build, modify, and reformat Markdown easily.
- **Render to HTML** — Convert Markdown AST to HTML.
- **Modular design** — You can disable parsing entirely and use only the AST types.

---

## 📦 Installation

Add the crate using Cargo:

```bash
cargo add markdown-ppp
```

If you want **only** the AST definitions without parsing functionality, disable default features manually:

```toml
[dependencies]
markdown-ppp = { version = "0.1.0", default-features = false }
```

---

## 🛠 Usage

### Parsing Markdown

The main entry point for parsing is the `parse_markdown` function, available at:

```rust
pub fn parse_markdown(
    state: MarkdownParserState,
    input: &str,
) -> Result<Document, nom::Err<nom::error::Error<&str>>>
```

Example:

```rust
use markdown_ppp::parse::parse_markdown;
use markdown_ppp::ast::Document;
use std::rc::Rc;

fn main() {
    let state = markdown_ppp::parse::MarkdownParserState::new();
    let input = "# Hello, World!";

    match parse_markdown(Rc::new(state), input) {
        Ok(document) => {
            println!("Parsed document: {:?}", document);
        }
        Err(err) => {
            eprintln!("Failed to parse Markdown: {:?}", err);
        }
    }
}
```

### MarkdownParserState

The `MarkdownParserState` controls parsing behavior and can be customized.

You can create a default state easily:

```rust
use markdown_ppp::parser::config::*;

let state = MarkdownParserState::default();
```

Alternatively, you can configure it manually by providing a `MarkdownParserConfig`:

```rust
use markdown_ppp::parser::config::*;

let config = MarkdownParserConfig::default()
    .with_block_blockquote_behavior(ElementBehavior::Ignore);

let ast = parse_markdown(MarkdownParserState::with_config(config), "hello world")?;
```

This allows you to control how certain Markdown elements are parsed or ignored.

---

## 🧩 Customizing the parsing behavior

You can control how individual Markdown elements are parsed at a fine-grained level using the [`MarkdownParserConfig`](https://docs.rs/markdown-ppp/latest/markdown_ppp/parser/config/struct.MarkdownParserConfig.html) API.

Each element type (block-level or inline-level) can be configured with an `ElementBehavior`:

```rust
pub enum ElementBehavior<ELT> {
    /// The parser will parse the element normally.
    Parse,

    /// The parser will ignore the element and not parse it. In this case, alternative
    /// parsers will be tried.
    Ignore,

    /// Parse the element but do not include it in the output.
    Skip,

    /// Parse the element and apply a custom function to it.
    Map(ElementMapFn<ELT>),

    /// Parse the element and apply a custom function to it which returns an array of elements.
    FlatMap(ElementFlatMapFn<ELT>),
}
```

These behaviors can be set via builder-style methods on the config. For example, to skip parsing of thematic breaks and transform blockquotes:

```rust
use markdown_ppp::parser::config::*;
use markdown_ppp::ast::Block;

let config = MarkdownParserConfig::default()
    .with_block_thematic_break_behavior(ElementBehavior::Skip)
    .with_block_blockquote_behavior(ElementBehavior::Map(|mut bq: Block| {
        // Example transformation: replace all blockquotes with empty paragraphs
        Block::Paragraph(vec![])
    }));

let ast = parse_markdown(MarkdownParserState::with_config(config), input)?;
```

This mechanism allows you to override, filter, or completely redefine how each Markdown element is treated during parsing, giving you deep control over the resulting AST.

### Registering custom parsers

You can also register your own custom block-level or inline-level parsers by providing parser functions via configuration. These parsers are executed before the built-in ones and can be used to support additional syntax or override behavior.

To register a custom block parser:

```rust
use markdown_ppp::parser::config::*;
use markdown_ppp::ast::Block;
use std::{rc::Rc, cell::RefCell};
use nom::IResult;

let custom_block: CustomBlockParserFn = Rc::new(RefCell::new(Box::new(|input: &str| {
    if input.starts_with("::note") {
        let block = Block::Paragraph(vec!["This is a note block".into()]);
        Ok(("", block))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
    }
})));

let config = MarkdownParserConfig::default()
    .with_custom_block_parser(custom_block);
```

Similarly, to register a custom inline parser:

```rust
use markdown_ppp::parser::config::*;
use markdown_ppp::ast::Inline;
use std::{rc::Rc, cell::RefCell};
use nom::IResult;

let custom_inline: CustomInlineParserFn = Rc::new(RefCell::new(Box::new(|input: &str| {
    if input.starts_with("@@") {
        Ok((&input[2..], Inline::Text("custom-inline".into())))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
    }
})));

let config = config.with_custom_inline_parser(custom_inline);
```

This extensibility allows you to integrate domain-specific syntax and behaviors into the Markdown parser while reusing the base logic and AST structure provided by `markdown-ppp`., filter, or completely redefine how each Markdown element is treated during parsing.

---

## 📄 AST structure

The complete Markdown Abstract Syntax Tree (AST) is defined inside the module `markdown_ppp::ast`.


The `Document` struct represents the root node, and from there you can traverse the full tree of blocks and inlines, such as headings, paragraphs, lists, emphasis, and more.

You can use the AST independently without the parsing functionality by disabling default features.

## 🖨️ Pretty-printing (AST → Markdown)

You can convert an AST (`Document`) back into a formatted Markdown string using the `render_markdown` function from the `printer` module.

This feature is enabled by default via the `printer` feature.

### Basic example

```rust
use markdown_ppp::printer::render_markdown;
use markdown_ppp::printer::config::Config;
use markdown_ppp::ast::Document;

// Assume you already have a parsed or constructed Document
let document = Document::default();

// Render it back to a Markdown string with default configuration
let markdown_output = render_markdown(&document, Config::default());

println!("{}", markdown_output);
```

This will format the Markdown with a default line width of 80 characters.

### Customizing output width

You can control the maximum width of lines in the generated Markdown by customizing the Config:

```rust
use markdown_ppp::printer::render_markdown;
use markdown_ppp::printer::config::Config;
use markdown_ppp::ast::Document;

// Set a custom maximum width, e.g., 120 characters
let config = Config::default().with_width(120);

let markdown_output = render_markdown(&Document::default(), config);

println!("{}", markdown_output);
```

This is useful if you want to control wrapping behavior or generate more compact or expanded Markdown documents.

## 🖨️ Pretty-printing (AST → HTML)

You can convert an AST (`Document`) back into a formatted HTML string using the `render_html` function from the `html_printer` module.

This feature is enabled by default via the `html-printer` feature.

### Basic example

```rust
use markdown_ppp::html_printer::render_html;
use markdown_ppp::html_printer::config::Config;
use markdown_ppp::ast::Document;

let config = Config::default();
let ast = crate::parser::parse_markdown(crate::parser::MarkdownParserState::default(), "# Hello, World!")
    .unwrap();

println!("{}", render_html(&ast, config));
```

---

## 🔧 Optional features

| Feature         | Description                                                        |
|:----------------|:-------------------------------------------------------------------|
| `parser`        | Enables Markdown parsing support. Enabled by default.              |
| `printer`       | Enables AST → Markdown string conversion. Enabled by default.      |
| `html-printer`  | Enables AST → HTML string conversion. Enabled by default.          |
| `ast-serde`     | Adds `Serialize` and `Deserialize` traits to all AST types via `serde`. Disabled by default. |

If you only need the AST types without parsing functionality, you can add the crate without default features:

```bash
cargo add --no-default-features markdown-ppp
```

If you want to disable Markdown generation (AST → Markdown string conversion), disable the `printer` feature manually:

```bash
cargo add markdown-ppp --no-default-features --features parser
```

---

## 📚 Documentation

- [API Docs on docs.rs](https://docs.rs/markdown-ppp)
- [AI-generated documentation](https://deepwiki.com/johnlepikhin/markdown-ppp)

---

## 📝 License

Licensed under the [MIT License](LICENSE).


````

`src/ast/mod.rs`:

````
//! Fully‑typed Abstract Syntax Tree (AST) for CommonMark + GitHub Flavored Markdown (GFM)
//! ------------------------------------------------------------------------------------
//! This module models every construct described in the **CommonMark 1.0 specification**
//! together with the widely‑used **GFM extensions**: tables, strikethrough, autolinks,
//! task‑list items and footnotes.
//!
//! The design separates **block‑level** and **inline‑level** nodes because parsers and
//! renderers typically operate on these tiers independently.
//!
//! ```text
//! Document ─┐
//!           └─ Block ─┐
//!                     ├─ Inline
//!                     └─ ...
//! ```

// ——————————————————————————————————————————————————————————————————————————
// Document root
// ——————————————————————————————————————————————————————————————————————————

/// Root of a Markdown document
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Document {
    /// Top‑level block sequence **in document order**.
    pub blocks: Vec<Block>,
}

// ——————————————————————————————————————————————————————————————————————————
// Block‑level nodes
// ——————————————————————————————————————————————————————————————————————————

/// Block‑level constructs in the order they appear in the CommonMark spec.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Block {
    /// Ordinary paragraph
    Paragraph(Vec<Inline>),

    /// ATX (`# Heading`) or Setext (`===`) heading
    Heading(Heading),

    /// Thematic break (horizontal rule)
    ThematicBreak,

    /// Block quote
    BlockQuote(Vec<Block>),

    /// List (bullet or ordered)
    List(List),

    /// Fenced or indented code block
    CodeBlock(CodeBlock),

    /// Raw HTML block
    HtmlBlock(String),

    /// Link reference definition.  Preserved for round‑tripping.
    Definition(LinkDefinition),

    /// Tables
    Table(Table),

    /// Footnote definition
    FootnoteDefinition(FootnoteDefinition),

    /// Empty block. This is used to represent skipped blocks in the AST.
    Empty,
}

/// Heading with level 1–6 and inline content.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Heading {
    /// Kind of heading (ATX or Setext) together with the level.
    pub kind: HeadingKind,

    /// Inlines that form the heading text (before trimming).
    pub content: Vec<Inline>,
}

/// Heading with level 1–6 and inline content.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HeadingKind {
    /// ATX heading (`# Heading`)
    Atx(u8),

    /// Setext heading (`===` or `---`)
    Setext(SetextHeading),
}

/// Setext heading with level and underline type.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SetextHeading {
    /// Setext heading with `=` underline
    Level1,

    /// Setext heading with `-` underline
    Level2,
}

// ——————————————————————————————————————————————————————————————————————————
// Lists
// ——————————————————————————————————————————————————————————————————————————

/// A list container — bullet or ordered.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct List {
    /// Kind of list together with additional semantic data (start index or
    /// bullet marker).
    pub kind: ListKind,

    /// List items in source order.
    pub items: Vec<ListItem>,
}

/// Specifies *what kind* of list we have.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ListKind {
    /// Ordered list (`1.`, `42.` …) with an *optional* explicit start number.
    Ordered(ListOrderedKindOptions),

    /// Bullet list (`-`, `*`, or `+`) together with the concrete marker.
    Bullet(ListBulletKind),
}

/// Specifies *what kind* of list we have.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ListOrderedKindOptions {
    /// Start index (1, 2, …) for ordered lists.
    pub start: u64,
}

/// Concrete bullet character used for a bullet list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ListBulletKind {
    /// `-` U+002D
    Dash,

    /// `*` U+002A
    Star,

    /// `+` U+002B
    Plus,
}

/// Item within a list.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ListItem {
    /// Task‑list checkbox state (GFM task‑lists). `None` ⇒ not a task list.
    pub task: Option<TaskState>,

    /// Nested blocks inside the list item.
    pub blocks: Vec<Block>,
}

/// State of a task‑list checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TaskState {
    /// Unchecked (GFM task‑list item)
    Incomplete,

    /// Checked (GFM task‑list item)
    Complete,
}

// ——————————————————————————————————————————————————————————————————————————
// Code blocks
// ——————————————————————————————————————————————————————————————————————————

/// Fenced or indented code block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodeBlock {
    /// Distinguishes indented vs fenced code and stores the *info string*.
    pub kind: CodeBlockKind,

    /// Literal text inside the code block **without** final newline trimming.
    pub literal: String,
}

/// The concrete kind of a code block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CodeBlockKind {
    /// Indented block (≥ 4 spaces or 1 tab per line).
    Indented,

    /// Fenced block with *optional* info string (language, etc.).
    Fenced { info: Option<String> },
}

// ——————————————————————————————————————————————————————————————————————————
// Link reference definitions
// ——————————————————————————————————————————————————————————————————————————

/// Link reference definition (GFM) with a label, destination and optional title.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkDefinition {
    /// Link label (acts as the *identifier*).
    pub label: Vec<Inline>,

    /// Link URL (absolute or relative) or email address.
    pub destination: String,

    /// Optional title (for links and images).
    pub title: Option<String>,
}

// ——————————————————————————————————————————————————————————————————————————
// Tables
// ——————————————————————————————————————————————————————————————————————————

/// A table is a collection of rows and columns with optional alignment.
/// The first row is the header row.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Table {
    /// Each row is a vector of *cells*; header row is **row 0**.
    pub rows: Vec<TableRow>,

    /// Column alignment; `alignments.len() == column_count`.
    pub alignments: Vec<Alignment>,
}

/// A table row is a vector of cells (columns).
pub type TableRow = Vec<TableCell>;

/// A table cell is a vector of inlines (text, links, etc.).
pub type TableCell = Vec<Inline>;

/// Specifies the alignment of a table cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Alignment {
    /// No alignment specified
    None,

    /// Left aligned
    #[default]
    Left,

    /// Right aligned
    Center,

    /// Right aligned
    Right,
}

// ——————————————————————————————————————————————————————————————————————————
// Footnotes
// ——————————————————————————————————————————————————————————————————————————

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FootnoteDefinition {
    /// Normalized label (without leading `^`).
    pub label: String,

    /// Footnote content (blocks).
    pub blocks: Vec<Block>,
}

// ——————————————————————————————————————————————————————————————————————————
// Inline‑level nodes
// ——————————————————————————————————————————————————————————————————————————

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Inline {
    /// Plain text (decoded entity references, preserved backslash escapes).
    Text(String),

    /// Hard line break
    LineBreak,

    /// Inline code span
    Code(String),

    /// Raw HTML fragment
    Html(String),

    /// Link to a destination with optional title.
    Link(Link),

    /// Reference link
    LinkReference(LinkReference),

    /// Image with optional title.
    Image(Image),

    /// Emphasis (`*` / `_`)
    Emphasis(Vec<Inline>),
    /// Strong emphasis (`**` / `__`)
    Strong(Vec<Inline>),
    /// Strikethrough (`~~`)
    Strikethrough(Vec<Inline>),

    /// Autolink (`<https://>` or `<mailto:…>`)
    Autolink(String),

    /// Footnote reference (`[^label]`)
    FootnoteReference(String),

    /// Empty element. This is used to represent skipped elements in the AST.
    Empty,
}

/// Re‑usable structure for links and images (destination + children).
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Link {
    /// Destination URL (absolute or relative) or email address.
    pub destination: String,

    /// Optional title (for links and images).
    pub title: Option<String>,

    /// Inline content (text, code, etc.) inside the link or image.
    pub children: Vec<Inline>,
}

/// Re‑usable structure for links and images (destination + children).
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Image {
    /// Image URL (absolute or relative).
    pub destination: String,

    /// Optional title.
    pub title: Option<String>,

    /// Alternative text.
    pub alt: String,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[cfg_attr(feature = "ast-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinkReference {
    /// Link label (acts as the *identifier*).
    pub label: Vec<Inline>,

    /// Link text
    pub text: Vec<Inline>,
}

````

`src/lib.rs`:

````
pub mod ast;

#[cfg(feature = "parser")]
pub mod parser;

#[cfg(feature = "printer")]
pub mod printer;

#[cfg(feature = "html-printer")]
pub mod html_printer;

````

`src/parser/blocks/blockquote.rs`:

````
use crate::ast::Block;
use crate::parser::util::*;
use crate::parser::MarkdownParserState;
use nom::{
    character::complete::char,
    multi::{many1, many_m_n},
    sequence::preceded,
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn blockquote<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Block>> {
    move |input: &'a str| {
        let prefix = preceded(many_m_n(0, 3, char(' ')), char('>'));

        let (input, lines) =
            many1(preceded(prefix, line_terminated(not_eof_or_eol0))).parse(input)?;
        let inner = lines.join("\n");

        let (_, inner) = many1(crate::parser::blocks::block(state.clone()))
            .parse(&inner)
            .map_err(|err| err.map_input(|_| input))?;

        let inner = inner.into_iter().flatten().collect();

        Ok((input, inner))
    }
}

````

`src/parser/blocks/code_block.rs`:

````
use crate::ast::{CodeBlock, CodeBlockKind};
use crate::parser::util::*;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{not, opt, peek, recognize, value},
    multi::{many0, many1, many_m_n},
    sequence::preceded,
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn code_block<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, CodeBlock> {
    move |input: &'a str| {
        alt((
            code_block_indented(state.clone()),
            code_block_fenced(state.clone()),
        ))
        .parse(input)
    }
}

pub(crate) fn code_block_indented<'a>(
    _state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, CodeBlock> {
    move |input: &'a str| {
        let line_parser = preceded(
            alt((value((), many_m_n(4, 4, char(' '))), value((), char('\t')))),
            line_terminated(not_eof_or_eol0),
        );

        let (input, lines) = many1(line_parser).parse(input)?;
        let literal = lines.join("\n");

        let code_block = CodeBlock {
            kind: CodeBlockKind::Indented,
            literal,
        };

        Ok((input, code_block))
    }
}

pub(crate) fn code_block_fenced<'a>(
    _state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, CodeBlock> {
    move |input: &'a str| {
        let (input, space_prefix) = many_m_n(0, 3, char(' ')).parse(input)?;
        let prefix_length = space_prefix.len();

        let (input, (fence, info)) = line_terminated((
            recognize(alt((
                many_m_n(3, usize::MAX, char('`')),
                many_m_n(3, usize::MAX, char('~')),
            ))),
            opt(recognize(not_eof_or_eol1)),
        ))
        .parse(input)?;
        let ending_fence = || {
            line_terminated((
                many_m_n(0, 3, char(' ')),
                tag(fence),
                many0(char(fence.chars().next().unwrap())),
            ))
        };

        let (input, lines) = many0(preceded(
            peek(not(ending_fence())),
            preceded(
                many_m_n(0, prefix_length, char(' ')),
                line_terminated(not_eof_or_eol0),
            ),
        ))
        .parse(input)?;
        let (input, _) = ending_fence().parse(input)?;

        let literal = lines.join("\n");
        let code_block = CodeBlock {
            kind: CodeBlockKind::Fenced {
                info: info.map(|v| v.to_owned()),
            },
            literal,
        };

        Ok((input, code_block))
    }
}

````

`src/parser/blocks/footnote_definition.rs`:

````
use crate::ast::FootnoteDefinition;
use crate::parser::util::{line_terminated, not_eof_or_eol1};
use crate::parser::MarkdownParserState;
use nom::character::complete::{char, none_of};
use nom::{
    bytes::complete::tag,
    combinator::{recognize, verify},
    multi::{many0, many1, many_m_n},
    sequence::preceded,
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn footnote_definition<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, FootnoteDefinition> {
    move |input: &'a str| {
        let (input, _) = many_m_n(0, 3, char(' ')).parse(input)?;
        let (input, _) = tag("[^").parse(input)?;
        let (input, label) = recognize(many1(verify(none_of("]"), |c| *c != ']'))).parse(input)?;
        let (input, _) = tag("]:").parse(input)?;
        let (input, _) = many_m_n(0, 3, char(' ')).parse(input)?;
        let (input, first_line) = line_terminated(not_eof_or_eol1).parse(input)?;
        let (input, rest_lines) = many0(preceded(
            many_m_n(3, 3, char(' ')),
            line_terminated(not_eof_or_eol1),
        ))
        .parse(input)?;

        let total_size = first_line.len() + rest_lines.len();
        let mut footnote_content = String::with_capacity(total_size);
        if !first_line.is_empty() {
            footnote_content.push_str(first_line)
        }
        for line in rest_lines {
            footnote_content.push('\n');
            footnote_content.push_str(line)
        }

        let (_, blocks) = many0(crate::parser::blocks::block(state.clone()))
            .parse(&footnote_content)
            .map_err(|err| err.map_input(|_| input))?;

        let blocks = blocks.into_iter().flatten().collect();

        let v = FootnoteDefinition {
            label: label.to_owned(),
            blocks,
        };

        Ok((input, v))
    }
}

````

`src/parser/blocks/heading.rs`:

````
use crate::ast::{Block, Heading, HeadingKind, SetextHeading};
use crate::parser::util::*;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    character::complete::{char, space0, space1},
    combinator::{opt, value},
    multi::{many1, many_m_n},
    sequence::{preceded, terminated},
    IResult, Parser,
};
use std::rc::Rc;

/// Parse headings in format:
///      ### Header text
pub(crate) fn heading_v1<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Heading> {
    move |input: &'a str| {
        let to_space_or_not_to_space = if state.config.allow_no_space_in_headings {
            space0
        } else {
            space1
        };

        let (input, (prefix, _, content)) = (
            many_m_n(1, 6, char('#')),
            to_space_or_not_to_space,
            line_terminated(not_eof_or_eol1),
        )
            .parse(input)?;

        let (_, content) = crate::parser::inline::inline_many0(state.clone()).parse(content)?;

        let heading = Heading {
            kind: HeadingKind::Atx(prefix.len() as u8),
            content,
        };

        Ok((input, heading))
    }
}

/// Parse headings in format:
///      Heading text
///      ====
pub(crate) fn heading_v2_or_paragraph<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Block> {
    move |input: &'a str| {
        let (input, (content, level)) = (
            crate::parser::blocks::paragraph::paragraph(state.clone(), true),
            opt(heading_v2_level(state.clone())),
        )
            .parse(input)?;

        if let Some(level) = level {
            let heading = Heading {
                kind: HeadingKind::Setext(level),
                content,
            };
            return Ok((input, Block::Heading(heading)));
        }

        Ok((input, Block::Paragraph(content)))
    }
}

pub(crate) fn heading_v2_level<'a>(
    _state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, SetextHeading> {
    move |input: &'a str| {
        let setext_parser = alt((
            value(SetextHeading::Level1, many1(char('='))),
            value(SetextHeading::Level2, many1(char('-'))),
        ));

        let r = line_terminated(preceded(
            many_m_n(0, 3, char(' ')),
            terminated(setext_parser, space0),
        ))
        .parse(input)?;

        Ok(r)
    }
}

````

`src/parser/blocks/html_block.rs`:

````
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{
        alpha1, alphanumeric1, anychar, char, line_ending, one_of, satisfy, space0, space1,
    },
    combinator::{eof, not, opt, peek, recognize, value, verify},
    multi::{many0, many1, many_m_n},
    sequence::{delimited, pair, preceded, terminated},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn html_block(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        alt((
            html_block1(state.clone()),
            html_block2(state.clone()),
            html_block3(state.clone()),
            html_block4(state.clone()),
            html_block5(state.clone()),
            html_block6(state.clone()),
            html_block7(state.clone()),
        ))
        .parse(input)
    }
}

fn html_block1(_state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        let tag_variant_parser = || {
            alt((
                tag_no_case("script"),
                tag_no_case("pre"),
                tag_no_case("style"),
            ))
        };

        let end_parser = || delimited(tag("</"), tag_variant_parser(), char('>'));

        preceded(
            many_m_n(0, 3, char(' ')),
            recognize((
                char('<'),
                tag_variant_parser(),
                alt((
                    value((), char(' ')),
                    value((), char('>')),
                    value((), line_ending),
                )),
                many0(pair(peek(not(end_parser())), anychar)),
                end_parser(),
            )),
        )
        .parse(input)
    }
}

fn html_block2(_state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        preceded(
            many_m_n(0, 3, char(' ')),
            recognize((
                tag("<!--"),
                many0(pair(peek(not(tag("-->"))), anychar)),
                tag("-->"),
            )),
        )
        .parse(input)
    }
}

fn html_block3(_state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        preceded(
            many_m_n(0, 3, char(' ')),
            recognize((
                tag("<?"),
                many0(pair(peek(not(tag("?>"))), anychar)),
                tag("?>"),
            )),
        )
        .parse(input)
    }
}

fn html_block4(_state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        preceded(
            many_m_n(0, 3, char(' ')),
            recognize((
                tag("<!"),
                satisfy(|c| c.is_ascii_uppercase()),
                many0(pair(peek(not(char('>'))), anychar)),
                tag(">"),
            )),
        )
        .parse(input)
    }
}

fn html_block5(_state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        preceded(
            many_m_n(0, 3, char(' ')),
            recognize((
                tag("<![CDATA["),
                many0(pair(peek(not(tag("]]>"))), anychar)),
                tag("]]>"),
            )),
        )
        .parse(input)
    }
}

fn html_block6(_state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        let tag_variant = alt((
            alt((
                tag_no_case("address"),
                tag_no_case("article"),
                tag_no_case("aside"),
                tag_no_case("base"),
                tag_no_case("basefont"),
                tag_no_case("blockquote"),
                tag_no_case("body"),
                tag_no_case("caption"),
                tag_no_case("center"),
                tag_no_case("col"),
                tag_no_case("colgroup"),
            )),
            alt((
                tag_no_case("dd"),
                tag_no_case("details"),
                tag_no_case("dialog"),
                tag_no_case("dir"),
                tag_no_case("div"),
                tag_no_case("dl"),
                tag_no_case("dt"),
                tag_no_case("fieldset"),
                tag_no_case("figcaption"),
                tag_no_case("figure"),
                tag_no_case("footer"),
                tag_no_case("form"),
                tag_no_case("frame"),
                tag_no_case("frameset"),
            )),
            alt((
                tag_no_case("h1"),
                tag_no_case("h2"),
                tag_no_case("h3"),
                tag_no_case("h4"),
                tag_no_case("h5"),
                tag_no_case("h6"),
                tag_no_case("head"),
                tag_no_case("header"),
                tag_no_case("hr"),
                tag_no_case("html"),
                tag_no_case("iframe"),
                tag_no_case("legend"),
            )),
            alt((
                tag_no_case("li"),
                tag_no_case("link"),
                tag_no_case("main"),
                tag_no_case("menu"),
                tag_no_case("menuitem"),
                tag_no_case("nav"),
                tag_no_case("noframes"),
                tag_no_case("ol"),
                tag_no_case("optgroup"),
                tag_no_case("option"),
                tag_no_case("p"),
                tag_no_case("param"),
            )),
            alt((
                tag_no_case("section"),
                tag_no_case("source"),
                tag_no_case("summary"),
                tag_no_case("table"),
                tag_no_case("tbody"),
                tag_no_case("td"),
                tag_no_case("tfoot"),
                tag_no_case("th"),
                tag_no_case("thead"),
                tag_no_case("title"),
                tag_no_case("tr"),
                tag_no_case("track"),
                tag_no_case("ul"),
            )),
        ));
        let end_parser = || {
            alt((
                value((), terminated(line_ending, (space0, line_ending))),
                value((), eof),
            ))
        };

        preceded(
            many_m_n(0, 3, char(' ')),
            recognize((
                alt((value((), tag("</")), value((), char('<')))),
                tag_variant,
                alt((
                    value((), char(' ')),
                    value((), line_ending),
                    value((), tag("/>")),
                    value((), char('>')),
                )),
                many0(pair(peek(not(end_parser())), anychar)),
                opt(line_ending),
            )),
        )
        .parse(input)
    }
}

fn html_block7(_state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        let end_parser = || {
            alt((
                value((), (line_ending, space0, line_ending)),
                value((), eof),
            ))
        };

        preceded(
            many_m_n(0, 3, char(' ')),
            recognize((
                alt((
                    complete_open_html_tag(&["script", "pre", "style"]),
                    complete_closing_html_tag,
                )),
                alt((value((), line_ending), value((), char(' ')))),
                many0(pair(peek(not(end_parser())), anychar)),
                end_parser(),
            )),
        )
        .parse(input)
    }
}

fn complete_open_html_tag(
    restricted_tags: &'static [&'static str],
) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input: &str| {
        recognize((
            char('<'),
            verify(html_tag_name, |s: &str| {
                !restricted_tags
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(s))
            }),
            many0(html_tag_attribute),
            space0,
            opt(char('/')),
            char('>'),
        ))
        .parse(input)
    }
}

fn complete_closing_html_tag(input: &str) -> IResult<&str, &str> {
    recognize((tag("</"), html_tag_name, space0, char('>'))).parse(input)
}

fn html_tag_name(input: &str) -> IResult<&str, &str> {
    recognize((
        alpha1,
        many0(alt((value((), char('-')), value((), alphanumeric1)))),
    ))
    .parse(input)
}

fn html_tag_attribute(input: &str) -> IResult<&str, &str> {
    recognize((
        space1,
        html_tag_attribute_name,
        opt(html_tag_attribute_value_specification),
    ))
    .parse(input)
}

fn html_tag_attribute_name(input: &str) -> IResult<&str, &str> {
    recognize((
        alt((value((), alpha1), value((), one_of("_:")))),
        many0(alt((value((), one_of("_.:-")), value((), alphanumeric1)))),
    ))
    .parse(input)
}

fn html_tag_attribute_value_specification(input: &str) -> IResult<&str, &str> {
    recognize((space0, char('='), space0, html_tag_attribute_value)).parse(input)
}

fn html_tag_attribute_value(input: &str) -> IResult<&str, &str> {
    alt((
        html_tag_attribute_value_unquoted,
        html_tag_attribute_value_single_quoted,
        html_tag_attribute_value_double_quoted,
    ))
    .parse(input)
}

fn html_tag_attribute_value_unquoted(input: &str) -> IResult<&str, &str> {
    recognize(many1(pair(
        peek(not(alt((value((), space1), value((), one_of("\"'=<>`")))))),
        anychar,
    )))
    .parse(input)
}

fn html_tag_attribute_value_single_quoted(input: &str) -> IResult<&str, &str> {
    recognize(delimited(
        char('\''),
        pair(peek(not(char('\''))), anychar),
        char('\''),
    ))
    .parse(input)
}

fn html_tag_attribute_value_double_quoted(input: &str) -> IResult<&str, &str> {
    recognize(delimited(
        char('"'),
        pair(peek(not(char('"'))), anychar),
        char('"'),
    ))
    .parse(input)
}

````

`src/parser/blocks/link_definition.rs`:

````
use super::eof_or_eol;
use crate::ast::LinkDefinition;
use crate::parser::link_util::{link_destination, link_label, link_title};
use crate::parser::MarkdownParserState;
use nom::character::complete::{char, line_ending, space0, space1};
use nom::{
    branch::alt,
    combinator::{opt, recognize, verify},
    multi::{many1, many_m_n},
    sequence::preceded,
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn link_definition<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, LinkDefinition> {
    move |input: &'a str| {
        let mut one_line_whitespace0 = (space0, opt(line_ending), space0);
        let one_line_whitespace1 = verify(
            recognize(many1(alt((line_ending, space1)))),
            |chars: &str| {
                let mut newlines = 0;
                for ch in chars.chars() {
                    if ch == '\n' {
                        newlines += 1;
                    }
                }
                newlines <= 1
            },
        );

        let (input, label) =
            preceded(many_m_n(0, 3, char(' ')), link_label(state.clone())).parse(input)?;
        let (input, _) = char(':').parse(input)?;
        let (input, _) = one_line_whitespace0.parse(input)?;
        let (input, destination) = link_destination.parse(input)?;
        let (input, title) = opt(preceded(one_line_whitespace1, link_title)).parse(input)?;
        let (input, _) = eof_or_eol.parse(input)?;

        let v = LinkDefinition {
            label,
            destination,
            title,
        };

        Ok((input, v))
    }
}

````

`src/parser/blocks/list.rs`:

````
use crate::ast::{ListBulletKind, ListItem, ListKind, ListOrderedKindOptions, TaskState};
use crate::parser::util::*;
use crate::parser::MarkdownParserState;
use nom::combinator::verify;
use nom::{
    branch::alt,
    character::complete::{char, one_of, space0},
    combinator::{map, not, opt, peek, recognize, value},
    multi::{many0, many1, many_m_n},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};
use std::rc::Rc;

fn list_item_task_state(input: &str) -> IResult<&str, TaskState> {
    delimited(
        char('['),
        alt((
            value(TaskState::Complete, one_of("xX")),
            value(TaskState::Incomplete, char(' ')),
        )),
        char(']'),
    )
    .parse(input)
}

fn list_marker(input: &str) -> IResult<&str, ListKind> {
    alt((
        list_marker_ordered,
        list_marker_star,
        list_marker_plus,
        list_marker_dash,
    ))
    .parse(input)
}

fn list_marker_star(input: &str) -> IResult<&str, ListKind> {
    map(char('*'), |_| ListKind::Bullet(ListBulletKind::Star)).parse(input)
}

fn list_marker_plus(input: &str) -> IResult<&str, ListKind> {
    map(char('+'), |_| ListKind::Bullet(ListBulletKind::Plus)).parse(input)
}

fn list_marker_dash(input: &str) -> IResult<&str, ListKind> {
    map(char('-'), |_| ListKind::Bullet(ListBulletKind::Dash)).parse(input)
}

fn list_marker_ordered(input: &str) -> IResult<&str, ListKind> {
    map(
        terminated(nom::character::complete::u64, one_of(".)")),
        |start| ListKind::Ordered(ListOrderedKindOptions { start }),
    )
    .parse(input)
}

fn list_marker_followed_by_spaces(
    input: &str,
) -> IResult<&str, (ListKind, usize, Option<TaskState>)> {
    let (remaining, kind) = delimited(
        many_m_n(0, 3, char(' ')),
        list_marker,
        many_m_n(1, 4, char(' ')),
    )
    .parse(input)?;

    let consumed = input.len() - remaining.len();

    let (input, task_state) = opt(terminated(list_item_task_state, char(' '))).parse(remaining)?;

    Ok((input, (kind, consumed, task_state)))
}

fn list_marker_followed_by_newline(
    input: &str,
) -> IResult<&str, (ListKind, usize, Option<TaskState>)> {
    let (remaining, kind) = preceded(many_m_n(0, 3, char(' ')), list_marker).parse(input)?;

    // Cases:
    // 1.
    // 1.____
    if let Ok((tail, _)) = line_terminated(space0).parse(remaining) {
        // Calculate prefix length: consumed + 1 space
        let consumed = input.len() - remaining.len() + 1;

        return Ok((tail, (kind, consumed, None)));
    }

    let (remaining, _) = many_m_n(0, 3, char(' ')).parse(remaining)?;
    let consumed = input.len() - remaining.len() + 1;

    let (remaining, task_state) = line_terminated(list_item_task_state).parse(remaining)?;

    Ok((remaining, (kind, consumed, Some(task_state))))
}

pub(crate) fn list_marker_with_span_size(
    input: &str,
) -> IResult<&str, (ListKind, usize, Option<TaskState>, String)> {
    alt((
        map(
            list_marker_followed_by_newline,
            |(list_kind, prefix_length, task_state)| {
                (list_kind, prefix_length, task_state, String::new())
            },
        ),
        (map(
            (
                list_marker_followed_by_spaces,
                line_terminated(not_eof_or_eol0),
            ),
            |((list_kind, prefix_length, task_state), s)| {
                (list_kind, prefix_length, task_state, s.to_string())
            },
        )),
    ))
    .parse(input)
}

fn list_item_rest_line(
    state: Rc<MarkdownParserState>,
    list_kind: ListKind,
    prefix_length: usize,
) -> impl FnMut(&str) -> IResult<&str, Vec<&str>> {
    move |input: &str| {
        // Stop parsing lines on EOF
        if input.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }

        let marker_parser = match list_kind {
            ListKind::Ordered(_) => list_marker_ordered,
            ListKind::Bullet(ListBulletKind::Star) => list_marker_star,
            ListKind::Bullet(ListBulletKind::Plus) => list_marker_plus,
            ListKind::Bullet(ListBulletKind::Dash) => list_marker_dash,
        };

        line_terminated(preceded(
            peek(not(alt((
                value(
                    (),
                    crate::parser::blocks::thematic_break::thematic_break(state.clone()),
                ),
                value(
                    (),
                    (
                        verify(
                            recognize(many_m_n(0, prefix_length, char(' '))),
                            |indent: &str| indent.len() < prefix_length,
                        ),
                        marker_parser,
                    ),
                ),
            )))),
            alt((
                // If starts with 0 <= prefix_length spaces
                preceded(
                    many_m_n(0, prefix_length, char(' ')),
                    map(not_eof_or_eol1, |v| vec![v]),
                ),
                // If this is empty line, followed by prefix_length spaces
                map(
                    (
                        recognize(many1(line_terminated(space0))),
                        preceded(
                            many_m_n(prefix_length, prefix_length, char(' ')),
                            not_eof_or_eol1,
                        ),
                    ),
                    |(newlines, content)| vec![newlines, content],
                ),
            )),
        ))
        .parse(input)
    }
}

fn list_item_lines(
    state: Rc<MarkdownParserState>,
    list_kind: ListKind,
    prefix_length: usize,
) -> impl FnMut(&str) -> IResult<&str, Vec<Vec<&str>>> {
    move |input: &str| {
        many0(list_item_rest_line(
            state.clone(),
            list_kind.clone(),
            prefix_length,
        ))
        .parse(input)
    }
}

pub(crate) fn list_item(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&str) -> IResult<&str, (ListKind, ListItem)> {
    move |input: &str| {
        let (input, (list_kind, item_prefix_length, task_state, first_line)) =
            list_marker_with_span_size(input)?;

        let (input, rest_lines) =
            list_item_lines(state.clone(), list_kind.clone(), item_prefix_length).parse(input)?;

        let total_size = first_line.len() + rest_lines.len();
        let mut item_content = String::with_capacity(total_size);
        if !first_line.is_empty() {
            item_content.push_str(&first_line)
        }
        for line in rest_lines {
            item_content.push('\n');
            for subline in line {
                item_content.push_str(subline)
            }
        }

        let (_, blocks) = many0(crate::parser::blocks::block(state.clone()))
            .parse(&item_content)
            .map_err(|err| err.map_input(|_| input))?;

        let blocks = blocks.into_iter().flatten().collect();

        let item = ListItem {
            task: task_state,
            blocks,
        };
        Ok((input, (list_kind, item)))
    }
}

pub(crate) fn list(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&str) -> IResult<&str, crate::ast::List> {
    move |input: &str| {
        let (input, items) = many1(list_item(state.clone())).parse(input)?;

        // With many1(), first element always present
        let first_item = items.first().unwrap();

        let list = crate::ast::List {
            kind: first_item.0.clone(),
            items: items.into_iter().map(|(_, item)| item).collect(),
        };

        Ok((input, list))
    }
}

````

`src/parser/blocks/mod.rs`:

````
mod blockquote;
mod code_block;
mod footnote_definition;
mod heading;
mod html_block;
mod link_definition;
mod list;
pub(crate) mod paragraph;
mod table;
mod thematic_break;

#[cfg(test)]
mod tests;

use crate::ast::Block;
use crate::parser::util::*;
use crate::parser::MarkdownParserState;
use nom::branch::alt;
use nom::combinator::fail;
use nom::{combinator::map, sequence::preceded, IResult, Parser};
use std::rc::Rc;

pub(crate) fn block<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Block>> {
    move |input: &'a str| {
        preceded(
            many_empty_lines0,
            alt((
                conditional_block(
                    state.config.block_code_block_behavior.clone(),
                    map(
                        crate::parser::blocks::code_block::code_block(state.clone()),
                        Block::CodeBlock,
                    ),
                ),
                conditional_block(
                    state.config.block_heading_v1_behavior.clone(),
                    map(
                        crate::parser::blocks::heading::heading_v1(state.clone()),
                        Block::Heading,
                    ),
                ),
                conditional_block(
                    state.config.block_heading_v2_behavior.clone(),
                    crate::parser::blocks::heading::heading_v2_or_paragraph(state.clone()),
                ),
                conditional_block(
                    state.config.block_thematic_break_behavior.clone(),
                    map(
                        crate::parser::blocks::thematic_break::thematic_break(state.clone()),
                        |()| Block::ThematicBreak,
                    ),
                ),
                conditional_block(
                    state.config.block_blockquote_behavior.clone(),
                    map(
                        crate::parser::blocks::blockquote::blockquote(state.clone()),
                        Block::BlockQuote,
                    ),
                ),
                conditional_block(
                    state.config.block_list_behavior.clone(),
                    map(
                        crate::parser::blocks::list::list(state.clone()),
                        Block::List,
                    ),
                ),
                conditional_block(
                    state.config.block_html_block_behavior.clone(),
                    map(
                        crate::parser::blocks::html_block::html_block(state.clone()),
                        |s| Block::HtmlBlock(s.to_owned()),
                    ),
                ),
                // Alway try before link definition
                conditional_block(
                    state.config.block_footnote_definition_behavior.clone(),
                    map(
                        crate::parser::blocks::footnote_definition::footnote_definition(
                            state.clone(),
                        ),
                        Block::FootnoteDefinition,
                    ),
                ),
                conditional_block(
                    state.config.block_link_definition_behavior.clone(),
                    map(
                        crate::parser::blocks::link_definition::link_definition(state.clone()),
                        Block::Definition,
                    ),
                ),
                conditional_block(
                    state.config.block_table_behavior.clone(),
                    map(
                        crate::parser::blocks::table::table(state.clone()),
                        Block::Table,
                    ),
                ),
                custom_parser(state.clone()),
                conditional_block(
                    state.config.block_paragraph_behavior.clone(),
                    map(
                        crate::parser::blocks::paragraph::paragraph(state.clone(), false),
                        Block::Paragraph,
                    ),
                ),
            )),
        )
        .parse(input)
    }
}

pub(crate) fn custom_parser(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&str) -> IResult<&str, Vec<Block>> {
    move |input: &str| {
        if let Some(custom_parser) = state.config.custom_block_parser.as_ref() {
            let mut p = (**custom_parser).borrow_mut();
            (p.as_mut())(input)
        } else {
            fail().parse(input)
        }
    }
}

````

`src/parser/blocks/paragraph.rs`:

````
use crate::ast::Inline;
use crate::parser::util::*;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    character::complete::{char, line_ending, space0},
    combinator::{not, peek, value},
    multi::{many_m_n, separated_list0},
    sequence::preceded,
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn paragraph<'a>(
    state: Rc<MarkdownParserState>,
    check_first_line: bool,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Inline>> {
    move |input: &'a str| {
        let mut lines = Vec::new();
        let input = if check_first_line {
            input
        } else {
            // Skip checks for the first line, just make it a paragraph
            let (input, first_line) =
                preceded(many_m_n(0, 3, char(' ')), not_eof_or_eol1).parse(input)?;
            lines.push(first_line);
            input
        };

        let paragraph_parser = separated_list0(
            line_ending,
            preceded(
                is_paragraph_line_start(state.clone()),
                preceded(many_m_n(0, 3, char(' ')), not_eof_or_eol1),
            ),
        );
        let (input, rest_lines) = line_terminated(paragraph_parser).parse(input)?;
        lines.extend(rest_lines);

        let content = lines.join("\n");

        let (_, content) = crate::parser::inline::inline_many1(state.clone())
            .parse(content.as_str())
            .map_err(|err| err.map_input(|_| input))?;

        Ok((input, content))
    }
}

pub(crate) fn is_paragraph_line_start<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |input: &'a str| {
        peek(not(alt((
            conditional_block_unit(
                state.config.block_heading_v1_behavior.clone(),
                value(
                    (),
                    crate::parser::blocks::heading::heading_v1(state.clone()),
                ),
            ),
            conditional_block_unit(
                state.config.block_heading_v2_behavior.clone(),
                value(
                    (),
                    crate::parser::blocks::heading::heading_v2_level(state.clone()),
                ),
            ),
            conditional_block_unit(
                state.config.block_thematic_break_behavior.clone(),
                crate::parser::blocks::thematic_break::thematic_break(state.clone()),
            ),
            conditional_block_unit(
                state.config.block_blockquote_behavior.clone(),
                value(
                    (),
                    crate::parser::blocks::blockquote::blockquote(state.clone()),
                ),
            ),
            conditional_block_unit(
                state.config.block_list_behavior.clone(),
                value((), crate::parser::blocks::list::list_item(state.clone())),
            ),
            conditional_block_unit(
                state.config.block_code_block_behavior.clone(),
                value(
                    (),
                    crate::parser::blocks::code_block::code_block_fenced(state.clone()),
                ),
            ),
            conditional_block_unit(
                state.config.block_html_block_behavior.clone(),
                value(
                    (),
                    crate::parser::blocks::html_block::html_block(state.clone()),
                ),
            ),
            conditional_block_unit(
                state.config.block_link_definition_behavior.clone(),
                value(
                    (),
                    crate::parser::blocks::link_definition::link_definition(state.clone()),
                ),
            ),
            conditional_block_unit(
                state.config.block_footnote_definition_behavior.clone(),
                value(
                    (),
                    crate::parser::blocks::footnote_definition::footnote_definition(state.clone()),
                ),
            ),
            conditional_block_unit(
                state.config.block_table_behavior.clone(),
                value((), crate::parser::blocks::table::table(state.clone())),
            ),
            value(
                vec![()],
                crate::parser::blocks::custom_parser(state.clone()),
            ),
            value(vec![()], line_terminated(space0)),
        ))))
        .parse(input)
    }
}

````

`src/parser/blocks/table.rs`:

````
use super::{eof_or_eol, line_terminated};
use crate::ast::{Alignment, Inline, Table, TableRow};
use crate::parser::MarkdownParserState;
use nom::multi::many_m_n;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, space0},
    combinator::{map, not, opt, recognize, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn table<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Table> {
    move |input: &'a str| {
        let (input, header) = parse_table_row(state.clone()).parse(input)?;
        let col_count = header.len();

        let (input, alignments) = parse_alignment_row.parse(input)?;
        if alignments.len() != col_count {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )));
        }

        let (input, rows) = parse_table_data_rows(state.clone(), col_count).parse(input)?;

        Ok((
            input,
            Table {
                rows: std::iter::once(header).chain(rows).collect(),
                alignments,
            },
        ))
    }
}

fn parse_table_data_rows<'a>(
    state: Rc<MarkdownParserState>,
    col_count: usize,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<TableRow>> {
    move |input: &'a str| {
        many0(map(parse_table_row(state.clone()), move |mut row| {
            match row.len().cmp(&col_count) {
                std::cmp::Ordering::Less => {
                    row.extend(
                        (0..(col_count - row.len())).map(|_| vec![Inline::Text(String::new())]),
                    );
                }
                std::cmp::Ordering::Greater => {
                    row.truncate(col_count);
                }
                _ => {}
            }
            row
        }))
        .parse(input)
    }
}

fn parse_alignment_row(input: &str) -> IResult<&str, Vec<Alignment>> {
    fn parse_cell_alignment(cell: &str) -> Alignment {
        let trimmed = cell.trim();
        let starts_with_colon = trimmed.starts_with(':');
        let ends_with_colon = trimmed.ends_with(':');

        match (starts_with_colon, ends_with_colon) {
            (true, true) => Alignment::Center,
            (true, false) => Alignment::Left,
            (false, true) => Alignment::Right,
            (false, false) => Alignment::None,
        }
    }

    let alignment_parser = delimited(
        space0,
        alt((
            recognize(delimited(char(':'), many1(char('-')), char(':'))),
            recognize(preceded(char(':'), many1(char('-')))),
            recognize(terminated(many1(char('-')), char(':'))),
            recognize(many1(char('-'))),
        )),
        space0,
    );

    line_terminated(preceded(
        many_m_n(0, 3, char(' ')),
        delimited(
            char('|'),
            separated_list1(char('|'), map(alignment_parser, parse_cell_alignment)),
            opt(char('|')),
        ),
    ))
    .parse(input)
}

fn parse_table_row<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, TableRow> {
    move |input: &'a str| {
        line_terminated(preceded(
            many_m_n(0, 3, char(' ')),
            delimited(
                char('|'),
                separated_list1(char('|'), cell_content(state.clone())),
                char('|'),
            ),
        ))
        .parse(input)
    }
}

fn cell_content<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Inline>> {
    move |input: &'a str| {
        let (input, chars) = many1(preceded(
            not(alt((value((), eof_or_eol), value((), char('|'))))),
            alt((value('|', tag("\\|")), anychar)),
        ))
        .parse(input)?;

        let content = chars.iter().collect::<String>();
        let trimmed_content = content.trim();
        let (_, content) = crate::parser::inline::inline_many0(state.clone())
            .parse(trimmed_content)
            .map_err(|err| err.map_input(|_| input))?;

        Ok((input, content))
    }
}

````

`src/parser/blocks/tests/blockquote.rs`:

````
use crate::ast::*;
use crate::parser::config::{ElementBehavior, MarkdownParserConfig};
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn blockquote1() {
    let doc = parse_markdown(MarkdownParserState::default(), "> a\n>\n>> b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::BlockQuote(vec![
                Block::Paragraph(vec![Inline::Text("a".to_owned())]),
                Block::BlockQuote(vec![Block::Paragraph(vec![Inline::Text("b".to_owned())])])
            ])]
        }
    );
}

#[test]
fn blockquote2() {
    let doc = parse_markdown(MarkdownParserState::default(), ">> a\n>>\n>> b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::BlockQuote(vec![Block::BlockQuote(vec![
                Block::Paragraph(vec![Inline::Text("a".to_owned()),]),
                Block::Paragraph(vec![Inline::Text("b".to_owned())])
            ])])]
        }
    );
}

#[test]
fn blockquote_skip1() {
    let config =
        MarkdownParserConfig::default().with_block_blockquote_behavior(ElementBehavior::Skip);
    let doc = parse_markdown(MarkdownParserState::with_config(config), "> a\n>\n>> b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Empty]
        }
    );
}

#[test]
fn blockquote_skip2() {
    let config =
        MarkdownParserConfig::default().with_block_blockquote_behavior(ElementBehavior::Skip);
    let doc = parse_markdown(MarkdownParserState::with_config(config), "a\n> a\n>\n>> b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::Paragraph(vec![Inline::Text("a".to_owned())]),
                Block::Empty
            ]
        }
    );
}

#[test]
fn blockquote_ignore1() {
    let config =
        MarkdownParserConfig::default().with_block_blockquote_behavior(ElementBehavior::Ignore);
    let doc = parse_markdown(MarkdownParserState::with_config(config), "> a\n>\n>> b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(
                "> a\n>\n>> b".to_owned()
            )]),]
        }
    );
}

#[test]
fn blockquote_ignore2() {
    let config =
        MarkdownParserConfig::default().with_block_blockquote_behavior(ElementBehavior::Ignore);
    let doc = parse_markdown(MarkdownParserState::with_config(config), "a\n> a\n>\n>> b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(
                "a\n> a\n>\n>> b".to_owned()
            )]),]
        }
    );
}

````

`src/parser/blocks/tests/code_block.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn code_block_indented1() {
    let doc = parse_markdown(MarkdownParserState::default(), "     a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::CodeBlock(CodeBlock {
                kind: CodeBlockKind::Indented,
                literal: " a".to_owned()
            })]
        }
    );
}

#[test]
fn code_block_indented2() {
    let doc = parse_markdown(MarkdownParserState::default(), "     a\n    b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::CodeBlock(CodeBlock {
                kind: CodeBlockKind::Indented,
                literal: " a\nb".to_owned()
            })]
        }
    );
}

#[test]
fn code_block_fenced1() {
    let doc = parse_markdown(MarkdownParserState::default(), "```\na\n```").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::CodeBlock(CodeBlock {
                kind: CodeBlockKind::Fenced { info: None },
                literal: "a".to_owned()
            })]
        }
    );
}

#[test]
fn code_block_fenced2() {
    let doc = parse_markdown(MarkdownParserState::default(), "  `````\na\n`````````").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::CodeBlock(CodeBlock {
                kind: CodeBlockKind::Fenced { info: None },
                literal: "a".to_owned()
            })]
        }
    );
}

#[test]
fn code_block_fenced3() {
    let doc = parse_markdown(MarkdownParserState::default(), "  ```\n    a\n      b\n```").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::CodeBlock(CodeBlock {
                kind: CodeBlockKind::Fenced { info: None },
                literal: "  a\n    b".to_owned()
            })]
        }
    );
}

#[test]
fn code_block_fenced4() {
    let doc = parse_markdown(MarkdownParserState::default(), "  ```rust\na\n```").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::CodeBlock(CodeBlock {
                kind: CodeBlockKind::Fenced {
                    info: Some("rust".to_owned())
                },
                literal: "a".to_owned()
            })]
        }
    );
}

````

`src/parser/blocks/tests/custom_parser.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};
use nom::combinator::value;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn custom_parser1() {
    use nom::Parser;
    let config = crate::parser::config::MarkdownParserConfig::default().with_custom_block_parser(
        Rc::new(RefCell::new(Box::new(|input: &str| {
            value(vec![Block::ThematicBreak], nom::bytes::complete::tag("///")).parse(input)
        }))),
    );
    let doc = parse_markdown(MarkdownParserState::with_config(config), "///\ntext\n===").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::ThematicBreak,
                Block::Heading(Heading {
                    kind: HeadingKind::Setext(SetextHeading::Level1),
                    content: vec![Inline::Text("text".to_owned())]
                })
            ]
        }
    );
}

````

`src/parser/blocks/tests/footnote_definition.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn footnote_definition1() {
    let doc = parse_markdown(MarkdownParserState::default(), "[^foo]: definition").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::FootnoteDefinition(FootnoteDefinition {
                label: "foo".to_owned(),
                blocks: vec![Block::Paragraph(vec![Inline::Text(
                    "definition".to_owned()
                )])]
            })]
        }
    );
}

#[test]
fn footnote_definition2() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "[^foo]: line1
    line2
",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::FootnoteDefinition(FootnoteDefinition {
                label: "foo".to_owned(),
                blocks: vec![Block::Paragraph(vec![Inline::Text(
                    "line1\nline2".to_owned()
                ),])]
            })]
        }
    );
}

````

`src/parser/blocks/tests/heading.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserConfig, MarkdownParserState};

#[test]
fn heading_v1() {
    let doc = parse_markdown(MarkdownParserState::default(), "## a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Heading(Heading {
                kind: HeadingKind::Atx(2),
                content: vec![Inline::Text("a".to_owned())]
            })]
        }
    );

    let doc = parse_markdown(MarkdownParserState::default(), "##a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("##a".to_string())])]
        }
    );

    let config = MarkdownParserConfig::default().with_allow_no_space_in_headings();
    let doc = parse_markdown(MarkdownParserState::with_config(config), "##a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Heading(Heading {
                kind: HeadingKind::Atx(2),
                content: vec![Inline::Text("a".to_owned())]
            })]
        }
    );
}

#[test]
fn heading_v2() {
    let doc = parse_markdown(MarkdownParserState::default(), "a\n==").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Heading(Heading {
                kind: HeadingKind::Setext(SetextHeading::Level1),
                content: vec![Inline::Text("a".to_owned())]
            })]
        }
    );

    let doc = parse_markdown(MarkdownParserState::default(), "a\n--").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Heading(Heading {
                kind: HeadingKind::Setext(SetextHeading::Level2),
                content: vec![Inline::Text("a".to_owned())]
            })]
        }
    );
}

````

`src/parser/blocks/tests/html_block.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn html_block1() {
    let doc = parse_markdown(MarkdownParserState::default(), "<script>\n</script>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock("<script>\n</script>".to_owned())]
        }
    );

    let doc = parse_markdown(
        MarkdownParserState::default(),
        "<script>\n\n<h1>hello</h1></script>",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock(
                "<script>\n\n<h1>hello</h1></script>".to_owned()
            )]
        }
    );
}

#[test]
fn html_block2() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "<!-- \n\nsome commented\n out code -->",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock(
                "<!-- \n\nsome commented\n out code -->".to_owned()
            )]
        }
    );
}

#[test]
fn html_block3() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "  <? \n\nsome \n   code ?>  ",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock("<? \n\nsome \n   code ?>".to_owned())]
        }
    );
}

#[test]
fn html_block4() {
    let doc = parse_markdown(MarkdownParserState::default(), "  <!A some \n\n\n text >  ").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock("<!A some \n\n\n text >".to_owned())]
        }
    );
}

#[test]
fn html_block5() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "  <![CDATA[ ]\n\n[[]]<> ]]> ",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock("<![CDATA[ ]\n\n[[]]<> ]]>".to_owned())]
        }
    );
}

#[test]
fn html_block6() {
    let doc = parse_markdown(MarkdownParserState::default(), "  <body  \n\n").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock("<body  \n".to_owned())]
        }
    );

    let doc = parse_markdown(
        MarkdownParserState::default(),
        "  <body a b=c d='e' f=\"g\" >\n</body>\n\n",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock(
                "<body a b=c d='e' f=\"g\" >\n</body>\n".to_owned()
            )]
        }
    );

    let doc = parse_markdown(MarkdownParserState::default(), "  </body> <p>\n</p>\n\n").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::HtmlBlock("</body> <p>\n</p>\n".to_owned())]
        }
    );
}

#[test]
fn html_block_skip1() {
    let config = crate::parser::config::MarkdownParserConfig::default()
        .with_block_html_block_behavior(crate::parser::config::ElementBehavior::Skip);
    let doc = parse_markdown(
        MarkdownParserState::with_config(config.clone()),
        "<script>\n</script>",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Empty]
        }
    );

    let doc = parse_markdown(
        MarkdownParserState::with_config(config),
        "<script>\n\n<h1>hello</h1></script>",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Empty]
        }
    );
}

#[test]
fn html_block_ignore1() {
    let config = crate::parser::config::MarkdownParserConfig::default()
        .with_block_html_block_behavior(crate::parser::config::ElementBehavior::Ignore);
    let doc = parse_markdown(
        MarkdownParserState::with_config(config.clone()),
        "<script>\n</script>",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(
                "<script>\n</script>".to_owned()
            )])]
        }
    );

    let doc = parse_markdown(
        MarkdownParserState::with_config(config),
        "<script>\n\n<h1>hello</h1></script>",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::Paragraph(vec![Inline::Text("<script>".to_owned())]),
                Block::Paragraph(vec![Inline::Text("<h1>hello</h1></script>".to_owned())])
            ]
        }
    );
}

````

`src/parser/blocks/tests/link_definition.rs`:

````
use crate::ast::*;
use crate::parser::config::{ElementBehavior, MarkdownParserConfig};
use crate::parser::{parse_markdown, MarkdownParserState};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn link_definition1() {
    let doc = parse_markdown(MarkdownParserState::default(), "[foo]: /url \"title\"").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![Inline::Text("foo".to_owned())],
                destination: "/url".to_owned(),
                title: Some("title".to_owned())
            })]
        }
    );
}

#[test]
fn link_definition2() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "   [foo]: 
      /url  
           'the title'
",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![Inline::Text("foo".to_owned())],
                destination: "/url".to_owned(),
                title: Some("the title".to_owned())
            })]
        }
    );
}

#[test]
fn link_definition3() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "[Foo*bar\\]]:my_(url) 'title (with parens)'",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![Inline::Text("Foo*bar]".to_owned())],
                destination: "my_(url)".to_owned(),
                title: Some("title (with parens)".to_owned())
            })]
        }
    );
}

#[test]
fn link_definition4() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "[Foo bar]:
<my url>
'title'",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![Inline::Text("Foo bar".to_owned())],
                destination: "my url".to_owned(),
                title: Some("title".to_owned())
            })]
        }
    );
}

#[test]
fn link_definition5() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "[foo]: /url '
title
line1
line2
'",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![Inline::Text("foo".to_owned())],
                destination: "/url".to_owned(),
                title: Some("\ntitle\nline1\nline2\n".to_owned())
            })]
        }
    );
}

#[test]
fn link_definition6() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "[foo]:
/url",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![Inline::Text("foo".to_owned())],
                destination: "/url".to_owned(),
                title: None
            })]
        }
    );
}

#[test]
fn link_definition7() {
    let doc = parse_markdown(MarkdownParserState::default(), "[foo]: <>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![Inline::Text("foo".to_owned())],
                destination: "".to_owned(),
                title: None
            })]
        }
    );
}

#[test]
fn link_definition_mapped1() {
    let config = MarkdownParserConfig::default().with_block_link_definition_behavior(
        ElementBehavior::Map(Rc::new(RefCell::new(Box::new(|block| {
            if let Block::Definition(v) = block {
                let mut label = vec![Inline::Text("mapped ".to_owned())];
                label.extend(v.label);
                Block::Definition(LinkDefinition {
                    label,
                    destination: format!("mapped {}", v.destination),
                    title: v.title.map(|t| format!("mapped {}", t)),
                })
            } else {
                block
            }
        })))),
    );
    let doc = parse_markdown(
        MarkdownParserState::with_config(config),
        "[foo]: /url \"title\"",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Definition(LinkDefinition {
                label: vec![
                    Inline::Text("mapped ".to_owned()),
                    Inline::Text("foo".to_owned())
                ],
                destination: "mapped /url".to_owned(),
                title: Some("mapped title".to_owned())
            })]
        }
    );
}

#[test]
fn link_definition_mapped2() {
    let config = MarkdownParserConfig::default().with_block_link_definition_behavior(
        ElementBehavior::FlatMap(Rc::new(RefCell::new(Box::new(|block| {
            if let Block::Definition(v) = block {
                let mut label = vec![Inline::Text("mapped ".to_owned())];
                label.extend(v.label);
                let link1 = Block::Definition(LinkDefinition {
                    label: label.clone(),
                    destination: format!("mapped {}", v.destination),
                    title: v.title.as_ref().map(|t| format!("mapped1 {}", t)),
                });
                let link2 = Block::Definition(LinkDefinition {
                    label,
                    destination: format!("mapped {}", v.destination),
                    title: v.title.map(|t| format!("mapped2 {}", t)),
                });
                vec![link1, link2]
            } else {
                vec![block]
            }
        })))),
    );
    let doc = parse_markdown(
        MarkdownParserState::with_config(config),
        "[foo]: /url \"title\"",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::Definition(LinkDefinition {
                    label: vec![
                        Inline::Text("mapped ".to_owned()),
                        Inline::Text("foo".to_owned())
                    ],
                    destination: "mapped /url".to_owned(),
                    title: Some("mapped1 title".to_owned())
                }),
                Block::Definition(LinkDefinition {
                    label: vec![
                        Inline::Text("mapped ".to_owned()),
                        Inline::Text("foo".to_owned())
                    ],
                    destination: "mapped /url".to_owned(),
                    title: Some("mapped2 title".to_owned())
                }),
            ]
        }
    );
}

````

`src/parser/blocks/tests/list.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn list1() {
    let doc = parse_markdown(MarkdownParserState::default(), "1. a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Ordered(ListOrderedKindOptions { start: 1 }),
                items: vec![ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        }
    );
}

#[test]
fn list2() {
    let doc = parse_markdown(MarkdownParserState::default(), "0100. a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Ordered(ListOrderedKindOptions { start: 100 }),
                items: vec![ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        }
    );
}

#[test]
fn list3() {
    let doc = parse_markdown(MarkdownParserState::default(), "1) a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Ordered(ListOrderedKindOptions { start: 1 }),
                items: vec![ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        }
    );
}

#[test]
fn list4() {
    let doc = parse_markdown(MarkdownParserState::default(), " -   a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        }
    );
}

#[test]
fn list5() {
    let doc = parse_markdown(MarkdownParserState::default(), "1. a\n2. b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Ordered(ListOrderedKindOptions { start: 1 }),
                items: vec![
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                    },
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("b".to_owned())])]
                    }
                ]
            })]
        }
    );
}

#[test]
fn list6() {
    let doc = parse_markdown(MarkdownParserState::default(), " - a\nb").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a\nb".to_owned())])]
                }]
            })]
        }
    );
}

#[test]
fn list7() {
    let doc = parse_markdown(MarkdownParserState::default(), " - a\nb\n\nc").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::List(List {
                    kind: ListKind::Bullet(ListBulletKind::Dash),
                    items: vec![ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("a\nb".to_owned())])]
                    }]
                }),
                Block::Paragraph(vec![Inline::Text("c".to_owned())])
            ]
        },
    );
}

#[test]
fn list8() {
    let doc = parse_markdown(MarkdownParserState::default(), " - a\nb\n\n   c").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: None,
                    blocks: vec![
                        Block::Paragraph(vec![Inline::Text("a\nb".to_owned())]),
                        Block::Paragraph(vec![Inline::Text("c".to_owned())]),
                    ]
                }]
            })]
        },
    );
}

#[test]
fn list9() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "1. list1\n   * list2\n   * list2\n2. list1",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Ordered(ListOrderedKindOptions { start: 1 }),
                items: vec![
                    ListItem {
                        task: None,
                        blocks: vec![
                            Block::Paragraph(vec![Inline::Text("list1".to_owned())]),
                            Block::List(List {
                                kind: ListKind::Bullet(ListBulletKind::Star),
                                items: vec![
                                    ListItem {
                                        task: None,
                                        blocks: vec![Block::Paragraph(vec![Inline::Text(
                                            "list2".to_owned()
                                        )]),]
                                    },
                                    ListItem {
                                        task: None,
                                        blocks: vec![Block::Paragraph(vec![Inline::Text(
                                            "list2".to_owned()
                                        )]),]
                                    }
                                ]
                            })
                        ]
                    },
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("list1".to_owned())])]
                    }
                ]
            })]
        },
    );
}

#[test]
fn list10() {
    let doc = parse_markdown(MarkdownParserState::default(), " * list1\n * list1").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Star),
                items: vec![
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("list1".to_owned())])]
                    },
                    ListItem {
                        task: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("list1".to_owned())])]
                    }
                ]
            })]
        },
    );
}

#[test]
fn task_list1() {
    let doc = parse_markdown(MarkdownParserState::default(), " - [ ] a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: Some(TaskState::Incomplete),
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        },
    );
}

#[test]
fn task_list2() {
    let doc = parse_markdown(MarkdownParserState::default(), " - [x] a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: Some(TaskState::Complete),
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        },
    );
}

#[test]
fn task_list3() {
    let doc = parse_markdown(MarkdownParserState::default(), " -   [x]   a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: Some(TaskState::Complete),
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        },
    );
}

#[test]
fn task_list4() {
    let doc = parse_markdown(MarkdownParserState::default(), " - [ ] ").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: Some(TaskState::Incomplete),
                    blocks: vec![]
                }]
            })]
        },
    );
}

#[test]
fn task_list5() {
    let doc = parse_markdown(MarkdownParserState::default(), "  -  [ ] \n\n     a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Dash),
                items: vec![ListItem {
                    task: Some(TaskState::Incomplete),
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_owned())])]
                }]
            })]
        },
    );
}

````

`src/parser/blocks/tests/mod.rs`:

````
mod blockquote;
mod code_block;
mod custom_parser;
mod footnote_definition;
mod heading;
mod html_block;
mod link_definition;
mod list;
mod paragraph;
mod table;
mod thematic_break;

````

`src/parser/blocks/tests/paragraph.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn minimal_paragraph() {
    let doc = parse_markdown(MarkdownParserState::default(), "a").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("a".to_string())])]
        }
    );

    let doc = parse_markdown(MarkdownParserState::default(), "a b c").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("a b c".to_string())])]
        }
    );

    let doc = parse_markdown(MarkdownParserState::default(), "a\nb\nc").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("a\nb\nc".to_string())])]
        }
    );
}

#[test]
fn multi_paragraph1() {
    let doc = parse_markdown(MarkdownParserState::default(), "a\n\nb").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::Paragraph(vec![Inline::Text("a".to_string())]),
                Block::Paragraph(vec![Inline::Text("b".to_string())]),
            ]
        }
    );
}

#[test]
fn multi_paragraph2() {
    let doc = parse_markdown(MarkdownParserState::default(), "a\n\n\n\n\nb").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::Paragraph(vec![Inline::Text("a".to_string())]),
                Block::Paragraph(vec![Inline::Text("b".to_string())]),
            ]
        }
    );
}

#[test]
fn multi_paragraph3() {
    let doc = parse_markdown(MarkdownParserState::default(), "a\n\n  b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::Paragraph(vec![Inline::Text("a".to_string())]),
                Block::Paragraph(vec![Inline::Text("b".to_string())]),
            ]
        }
    );
}

#[test]
fn multi_paragraph4() {
    let doc = parse_markdown(MarkdownParserState::default(), "aaa\n\n===").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![
                Block::Paragraph(vec![Inline::Text("aaa".to_string())]),
                Block::Paragraph(vec![Inline::Text("===".to_string())]),
            ]
        }
    );
}

#[test]
fn paragraph_with_indented_line1() {
    let doc = parse_markdown(MarkdownParserState::default(), "a\n    b").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("a\n b".to_string())])],
        }
    );
}

````

`src/parser/blocks/tests/table.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn table1() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "| foo | bar |
| --- | --- |
| baz | bim |",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Table(Table {
                rows: vec![
                    vec![
                        vec![Inline::Text("foo".to_owned())],
                        vec![Inline::Text("bar".to_owned())]
                    ],
                    vec![
                        vec![Inline::Text("baz".to_owned())],
                        vec![Inline::Text("bim".to_owned())]
                    ]
                ],
                alignments: vec![Alignment::None, Alignment::None]
            })]
        }
    );
}

#[test]
fn table2() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "| foo | bar |
| :-- | --: |
| baz | bim |",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Table(Table {
                rows: vec![
                    vec![
                        vec![Inline::Text("foo".to_owned())],
                        vec![Inline::Text("bar".to_owned())]
                    ],
                    vec![
                        vec![Inline::Text("baz".to_owned())],
                        vec![Inline::Text("bim".to_owned())]
                    ]
                ],
                alignments: vec![Alignment::Left, Alignment::Right]
            })]
        }
    );
}

#[test]
fn table3() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "| foo | bar |
| --- | --- |
| baz | b\\|im |",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Table(Table {
                rows: vec![
                    vec![
                        vec![Inline::Text("foo".to_owned())],
                        vec![Inline::Text("bar".to_owned())]
                    ],
                    vec![
                        vec![Inline::Text("baz".to_owned())],
                        vec![Inline::Text("b|im".to_owned())]
                    ]
                ],
                alignments: vec![Alignment::None, Alignment::None]
            })]
        }
    );
}

#[test]
fn table4() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "| abc | def |
| --- | --- |
| bar |
| bar | baz | boo |",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Table(Table {
                rows: vec![
                    vec![
                        vec![Inline::Text("abc".to_owned())],
                        vec![Inline::Text("def".to_owned())]
                    ],
                    vec![
                        vec![Inline::Text("bar".to_owned())],
                        vec![Inline::Text("".to_owned())],
                    ],
                    vec![
                        vec![Inline::Text("bar".to_owned())],
                        vec![Inline::Text("baz".to_owned())],
                    ]
                ],
                alignments: vec![Alignment::None, Alignment::None]
            })]
        }
    );
}

````

`src/parser/blocks/tests/thematic_break.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn thematic_break() {
    let doc = parse_markdown(MarkdownParserState::default(), "---").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::ThematicBreak]
        }
    );
}

````

`src/parser/blocks/thematic_break.rs`:

````
use crate::parser::util::*;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    character::complete::{char, space0},
    combinator::map,
    multi::{many, many_m_n},
    sequence::{preceded, terminated},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn thematic_break<'a>(
    _state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |input: &str| {
        map(
            line_terminated(preceded(
                many_m_n(0, 3, char(' ')),
                terminated(
                    alt((
                        many(3.., char('-')),
                        many(3.., char('_')),
                        many(3.., char('*')),
                    )),
                    space0,
                ),
            )),
            |_: Vec<_>| (),
        )
        .parse(input)
    }
}

````

`src/parser/config.rs`:

````
use nom::IResult;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Function type for mapping elements.
type ElementMapFn<ELT> = Rc<RefCell<Box<dyn FnMut(ELT) -> ELT>>>;

/// Function type for mapping elements.
type ElementFlatMapFn<ELT> = Rc<RefCell<Box<dyn FnMut(ELT) -> Vec<ELT>>>>;

/// Function type for custom block parsers.
type CustomBlockParserFn =
    Rc<RefCell<Box<dyn for<'a> FnMut(&'a str) -> IResult<&'a str, Vec<crate::ast::Block>>>>>;

/// Function type for custom inline parsers.
type CustomInlineParserFn =
    Rc<RefCell<Box<dyn for<'a> FnMut(&'a str) -> IResult<&'a str, Vec<crate::ast::Inline>>>>>;

/// Behavior of the parser when encountering certain elements.
#[derive(Clone)]
pub enum ElementBehavior<ELT> {
    /// The parser will parse the element normally.
    Parse,

    /// The parser will ignore the element and not parse it. In this case, alternative
    /// parsers will be tried.
    Ignore,

    /// Parse element but do not include it in the output.
    Skip,

    /// Parse the element and apply a custom function to it.
    Map(ElementMapFn<ELT>),

    /// Parse the element and apply a custom function to it which returns an array of elements.
    FlatMap(ElementFlatMapFn<ELT>),
}

/// A configuration for the Markdown parser.
#[derive(Clone)]
pub struct MarkdownParserConfig {
    /// If true, the parser will allow headings without a space after the hash marks.
    pub(crate) allow_no_space_in_headings: bool,

    /// A map of HTML entities to their corresponding `Entity` structs.
    pub(crate) html_entities_map: HashMap<String, &'static entities::Entity>,

    /// The behavior of the parser when encountering blockquotes.
    pub(crate) block_blockquote_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering headings in style 1 (e.g., `# Heading`).
    pub(crate) block_heading_v1_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering headings in style 2 (e.g., `Heading\n===`).
    pub(crate) block_heading_v2_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering thematic breaks (e.g., `---`).
    pub(crate) block_thematic_break_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering lists.
    pub(crate) block_list_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering code blocks.
    pub(crate) block_code_block_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering HTML blocks.
    pub(crate) block_html_block_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering footnote definitions.
    pub(crate) block_footnote_definition_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering link definitions.
    pub(crate) block_link_definition_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering tables.
    pub(crate) block_table_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering block paragraphs.
    pub(crate) block_paragraph_behavior: ElementBehavior<crate::ast::Block>,

    /// The behavior of the parser when encountering inline autolinks.
    pub(crate) inline_autolink_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline links.
    pub(crate) inline_link_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline footnote references.
    pub(crate) inline_footnote_reference_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline reference links.
    pub(crate) inline_reference_link_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline hard newlines.
    pub(crate) inline_hard_newline_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline images.
    pub(crate) inline_image_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline code spans.
    pub(crate) inline_code_span_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline emphasis.
    pub(crate) inline_emphasis_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline strikethrough.
    pub(crate) inline_strikethrough_behavior: ElementBehavior<crate::ast::Inline>,

    /// The behavior of the parser when encountering inline text.
    pub(crate) inline_text_behavior: ElementBehavior<crate::ast::Inline>,

    /// A custom parser for blocks. This is a function that takes a string and returns a `Block`.
    pub(crate) custom_block_parser: Option<CustomBlockParserFn>,

    /// A custom parser for inlines. This is a function that takes a string and returns a `Inline`.
    pub(crate) custom_inline_parser: Option<CustomInlineParserFn>,
}

impl Default for MarkdownParserConfig {
    fn default() -> Self {
        Self {
            allow_no_space_in_headings: false,
            html_entities_map: Self::make_html_entities_map(),
            block_blockquote_behavior: ElementBehavior::Parse,
            block_heading_v1_behavior: ElementBehavior::Parse,
            block_heading_v2_behavior: ElementBehavior::Parse,
            block_thematic_break_behavior: ElementBehavior::Parse,
            block_list_behavior: ElementBehavior::Parse,
            block_code_block_behavior: ElementBehavior::Parse,
            block_html_block_behavior: ElementBehavior::Parse,
            block_footnote_definition_behavior: ElementBehavior::Parse,
            block_link_definition_behavior: ElementBehavior::Parse,
            block_table_behavior: ElementBehavior::Parse,
            block_paragraph_behavior: ElementBehavior::Parse,
            inline_autolink_behavior: ElementBehavior::Parse,
            inline_link_behavior: ElementBehavior::Parse,
            inline_footnote_reference_behavior: ElementBehavior::Parse,
            inline_reference_link_behavior: ElementBehavior::Parse,
            inline_hard_newline_behavior: ElementBehavior::Parse,
            inline_image_behavior: ElementBehavior::Parse,
            inline_code_span_behavior: ElementBehavior::Parse,
            inline_emphasis_behavior: ElementBehavior::Parse,
            inline_strikethrough_behavior: ElementBehavior::Parse,
            inline_text_behavior: ElementBehavior::Parse,
            custom_block_parser: None,
            custom_inline_parser: None,
        }
    }
}

impl MarkdownParserConfig {
    fn make_html_entities_map() -> HashMap<String, &'static entities::Entity> {
        let mut map = HashMap::new();
        for entity in entities::ENTITIES.iter() {
            map.insert(entity.entity.to_string(), entity);
        }
        map
    }

    /// Enable the parser to allow headings without a space after the hash marks.
    pub fn with_allow_no_space_in_headings(self) -> Self {
        Self {
            allow_no_space_in_headings: true,
            ..self
        }
    }

    /// Set a custom map of HTML entities.
    pub fn with_html_entities_map(
        self,
        html_entities_map: HashMap<String, &'static entities::Entity>,
    ) -> Self {
        Self {
            html_entities_map,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering blockquotes.
    pub fn with_block_blockquote_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_blockquote_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering headings in style 1 (e.g., `# Heading`).
    pub fn with_block_heading_v1_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_heading_v1_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering headings in style 2 (e.g., `Heading\n===`).
    pub fn with_block_heading_v2_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_heading_v2_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering thematic breaks (e.g., `---`).
    pub fn with_block_thematic_break_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_thematic_break_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering lists.
    pub fn with_block_list_behavior(self, behavior: ElementBehavior<crate::ast::Block>) -> Self {
        Self {
            block_list_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering code blocks.
    pub fn with_block_code_block_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_code_block_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering HTML blocks.
    pub fn with_block_html_block_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_html_block_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering footnote definitions.
    pub fn with_block_footnote_definition_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_footnote_definition_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering link definitions.
    pub fn with_block_link_definition_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_link_definition_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering tables.
    pub fn with_block_table_behavior(self, behavior: ElementBehavior<crate::ast::Block>) -> Self {
        Self {
            block_table_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering block paragraphs.
    pub fn with_block_paragraph_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Block>,
    ) -> Self {
        Self {
            block_paragraph_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline autolinks.
    pub fn with_inline_autolink_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Inline>,
    ) -> Self {
        Self {
            inline_autolink_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline links.
    pub fn with_inline_link_behavior(self, behavior: ElementBehavior<crate::ast::Inline>) -> Self {
        Self {
            inline_link_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline footnote references.
    pub fn with_inline_footnote_reference_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Inline>,
    ) -> Self {
        Self {
            inline_footnote_reference_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline reference links.
    pub fn with_inline_reference_link_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Inline>,
    ) -> Self {
        Self {
            inline_reference_link_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline hard newlines.
    pub fn with_inline_hard_newline_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Inline>,
    ) -> Self {
        Self {
            inline_hard_newline_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline images.
    pub fn with_inline_image_behavior(self, behavior: ElementBehavior<crate::ast::Inline>) -> Self {
        Self {
            inline_image_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline code spans.
    pub fn with_inline_code_span_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Inline>,
    ) -> Self {
        Self {
            inline_code_span_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline emphasis.
    pub fn with_inline_emphasis_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Inline>,
    ) -> Self {
        Self {
            inline_emphasis_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline strikethrough.
    pub fn with_inline_strikethrough_behavior(
        self,
        behavior: ElementBehavior<crate::ast::Inline>,
    ) -> Self {
        Self {
            inline_strikethrough_behavior: behavior,
            ..self
        }
    }

    /// Set the behavior of the parser when encountering inline text.
    pub fn with_inline_text_behavior(self, behavior: ElementBehavior<crate::ast::Inline>) -> Self {
        Self {
            inline_text_behavior: behavior,
            ..self
        }
    }

    /// Set a custom parser for blocks.
    pub fn with_custom_block_parser(self, parser: CustomBlockParserFn) -> Self {
        Self {
            custom_block_parser: Some(parser),
            ..self
        }
    }

    /// Set a custom parser for inlines.
    pub fn with_custom_inline_parser(self, parser: CustomInlineParserFn) -> Self {
        Self {
            custom_inline_parser: Some(parser),
            ..self
        }
    }
}

````

`src/parser/inline/autolink.rs`:

````
use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, satisfy},
    combinator::{map, recognize},
    sequence::{delimited, pair, terminated},
    IResult, Parser,
};

pub(crate) fn autolink(input: &str) -> IResult<&str, String> {
    delimited(char('<'), alt((uri, email)), char('>')).parse(input)
}

/// uri: scheme ":" [^<>\u0000-\u0020]*
fn uri(input: &str) -> IResult<&str, String> {
    map(
        pair(
            terminated(scheme, char(':')),
            take_while(|c: char| {
                !c.is_ascii_control() && !c.is_ascii_whitespace() && c != '<' && c != '>'
            }),
        ),
        |(scheme_part, rest): (&str, &str)| {
            let mut s = String::from(scheme_part);
            s.push(':');
            s.push_str(rest);
            s
        },
    )
    .parse(input)
}

/// email: simplified form for <user@example.com>
fn email(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            take_while1(|c: char| c.is_ascii_alphanumeric() || "_-.".contains(c)),
            pair(
                char('@'),
                take_while1(|c: char| c.is_ascii_alphanumeric() || ".-".contains(c)),
            ),
        )),
        |v: &str| v.to_string(),
    )
    .parse(input)
}

/// scheme: [a-zA-Z][a-zA-Z0-9+.-]{1,31}
fn scheme(input: &str) -> IResult<&str, &str> {
    recognize(pair(satisfy(is_scheme_start), take_while1(is_scheme_char))).parse(input)
}

fn is_scheme_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')
}

fn is_scheme_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

````

`src/parser/inline/code_span.rs`:

````
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, line_ending, space0},
    combinator::{not, peek, recognize, value},
    multi::many1,
    sequence::preceded,
    IResult, Parser,
};

pub(crate) fn code_span(input: &str) -> IResult<&str, String> {
    let (input, open_ticks) = backtick_string(input)?;
    let tick_count = open_ticks.len();
    let closing_tag_value = "`".repeat(tick_count);

    let not_a_closing_tag = (tag(closing_tag_value.as_str()), char('`'));
    let closing_tag = preceded(
        peek(not(not_a_closing_tag)),
        tag(closing_tag_value.as_str()),
    );
    let empty_line = (line_ending, space0, line_ending);
    let content_parser = preceded(
        peek(not(alt((value((), closing_tag), value((), empty_line))))),
        anychar,
    );

    let (input, content) = recognize(many1(content_parser)).parse(input)?;
    let (input, _) = tag(closing_tag_value.as_str()).parse(input)?;

    let mut content = content.replace("\r\n", " ").replace("\n", " ");
    if content.starts_with(' ') && content.ends_with(' ') && content.trim() != "" {
        content = content[1..content.len() - 1].to_string();
    }

    Ok((input, content))
}

fn backtick_string(input: &str) -> IResult<&str, &str> {
    recognize(many1(char('`'))).parse(input)
}

````

`src/parser/inline/emphasis.rs`:

````
use crate::ast::Inline;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::anychar,
    combinator::{map, map_opt, not, peek, recognize, value, verify},
    multi::many1,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn emphasis(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&str) -> IResult<&str, Inline> {
    move |input: &str| {
        alt((
            map(
                alt((
                    delimited(
                        open_tag("***"),
                        emphasis_content(state.clone(), close_tag("***")),
                        close_tag("***"),
                    ),
                    delimited(
                        open_tag("___"),
                        emphasis_content(state.clone(), close_tag("___")),
                        close_tag("___"),
                    ),
                )),
                |inner| Inline::Strong(vec![Inline::Emphasis(inner)]),
            ),
            map(
                alt((
                    delimited(
                        open_tag("**"),
                        emphasis_content(state.clone(), close_tag("**")),
                        close_tag("**"),
                    ),
                    delimited(
                        open_tag("__"),
                        emphasis_content(state.clone(), close_tag("__")),
                        close_tag("__"),
                    ),
                )),
                Inline::Strong,
            ),
            map(
                alt((
                    delimited(
                        open_tag("*"),
                        emphasis_content(state.clone(), close_tag("*")),
                        close_tag("*"),
                    ),
                    delimited(
                        open_tag("_"),
                        emphasis_content(state.clone(), close_tag("_")),
                        close_tag("_"),
                    ),
                )),
                Inline::Emphasis,
            ),
        ))
        .parse(input)
    }
}

fn emphasis_content<'a, P>(
    state: Rc<MarkdownParserState>,
    mut close_tag: P,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Inline>>
where
    P: Parser<&'a str, Output = (), Error = nom::error::Error<&'a str>>,
{
    move |input: &str| {
        let not_end = |i: &'a str| close_tag.parse(i);
        map_opt(
            recognize(many1(preceded(
                peek(not(not_end)),
                alt((value((), tag("\\*")), value((), anychar))),
            ))),
            |content: &str| {
                crate::parser::inline::inline_many1(state.clone())
                    .parse(content)
                    .map(|(_, content)| content)
                    .ok()
            },
        )
        .parse(input)
    }
}

fn open_tag(tag_value: &'static str) -> impl FnMut(&str) -> IResult<&str, ()> {
    move |input: &str| {
        value(
            (),
            verify(tag(tag_value), |v: &str| {
                can_open(v.chars().next().unwrap(), input.chars().nth(v.len()))
            }),
        )
        .parse(input)
    }
}

fn can_open(marker: char, next: Option<char>) -> bool {
    let left_flanking = next.is_some_and(|c| !c.is_whitespace())
        && (next.is_some_and(|c| !is_punctuation(c)) || (next.is_some_and(is_punctuation)));
    if !left_flanking {
        return false;
    }
    if marker == '_' {
        let right_flanking = next.is_none_or(|c| c.is_whitespace() || is_punctuation(c));
        return !right_flanking;
    }
    true
}

fn close_tag(tag_value: &'static str) -> impl FnMut(&str) -> IResult<&str, ()> {
    move |input: &str| {
        value(
            (),
            verify(tag(tag_value), |v: &str| {
                can_close(v.chars().next().unwrap(), input.chars().nth(v.len()))
            }),
        )
        .parse(input)
    }
}

fn can_close(marker: char, next: Option<char>) -> bool {
    let right_flanking = next.is_none_or(|c| c.is_whitespace() || is_punctuation(c));
    if !right_flanking {
        return false;
    }

    if marker == '_' {
        let left_flanking = next.is_some_and(|c| !c.is_whitespace())
            && (next.is_some_and(|c| !is_punctuation(c)))
            || (next.is_some_and(is_punctuation));
        return !left_flanking || next.is_some_and(is_punctuation);
    }
    true
}

fn is_punctuation(c: char) -> bool {
    use unicode_categories::UnicodeCategories;
    c.is_ascii_punctuation() || c.is_punctuation()
}

````

`src/parser/inline/footnote_reference.rs`:

````
use crate::ast::Inline;
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, char},
    combinator::map,
    sequence::delimited,
    IResult, Parser,
};

pub(crate) fn footnote_reference<'a>(input: &'a str) -> IResult<&'a str, Inline> {
    map(
        delimited(tag("[^"), alphanumeric1, char(']')),
        |s: &'a str| Inline::FootnoteReference(s.to_owned()),
    )
    .parse(input)
}

````

`src/parser/inline/hard_newline.rs`:

````
use crate::ast::Inline;
use nom::multi::many_m_n;
use nom::{
    branch::alt,
    character::complete::{char, line_ending},
    combinator::value,
    sequence::pair,
    IResult, Parser,
};

pub(crate) fn hard_newline(input: &str) -> IResult<&str, Inline> {
    value(
        Inline::LineBreak,
        alt((
            value((), pair(char('\\'), line_ending)),
            value((), pair(many_m_n(2, usize::MAX, char(' ')), line_ending)),
        )),
    )
    .parse(input)
}

````

`src/parser/inline/html_entity.rs`:

````
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, hex_digit1, one_of},
    combinator::{map, map_opt, recognize},
    sequence::{delimited, preceded},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn html_entity(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&str) -> IResult<&str, String> {
    move |input: &str| alt((html_entity_alpha(state.clone()), html_entity_numeric)).parse(input)
}

fn html_entity_alpha(state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, String> {
    move |input: &str| {
        map_opt(recognize((char('&'), alpha1, char(';'))), |s: &str| {
            state
                .as_ref()
                .config
                .html_entities_map
                .get(s)
                .map(|entity| entity.characters.to_owned())
        })
        .parse(input)
    }
}

fn html_entity_numeric(input: &str) -> IResult<&str, String> {
    let base16 = map_opt(preceded(one_of("xX"), hex_digit1), |s: &str| {
        u32::from_str_radix(s, 16).ok()
    });
    let base10 = map_opt(digit1, |s: &str| s.parse::<u32>().ok());

    map(
        map_opt(
            delimited(tag("&#"), alt((base10, base16)), char(';')),
            char::from_u32,
        ),
        |c: char| c.to_string(),
    )
    .parse(input)
}

````

`src/parser/inline/image.rs`:

````
use crate::parser::link_util::link_title;
use crate::parser::MarkdownParserState;
use crate::{
    ast::{Image, Inline},
    parser::link_util::link_destination,
};
use nom::{
    bytes::complete::take_while1,
    character::complete::{char, multispace0},
    combinator::opt,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use std::rc::Rc;

// ![alt text](/url "title")
pub(crate) fn image<'a>(
    _state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Inline> {
    move |input: &'a str| {
        let (input, alt) = preceded(
            char('!'),
            delimited(char('['), take_while1(|c| c != ']'), char(']')),
        )
        .parse(input)?;

        let (input, (destination, title)) = delimited(
            char('('),
            (
                preceded(multispace0, link_destination),
                opt(preceded(multispace0, link_title)),
            ),
            preceded(multispace0, char(')')),
        )
        .parse(input)?;

        Ok((
            input,
            Inline::Image(Image {
                destination,
                title,
                alt: alt.to_owned(),
            }),
        ))
    }
}

````

`src/parser/inline/inline_link.rs`:

````
use crate::ast::Link;
use crate::parser::link_util::{link_destination, link_label, link_title};
use crate::parser::MarkdownParserState;
use nom::{
    character::complete::{char, multispace0},
    combinator::opt,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn inline_link<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Link> {
    move |input: &'a str| {
        let (input, (children, (destination, title))) = (
            link_label(state.clone()),
            delimited(
                char('('),
                (
                    preceded(multispace0, link_destination),
                    opt(preceded(multispace0, link_title)),
                ),
                preceded(multispace0, char(')')),
            ),
        )
            .parse(input)?;

        let link = Link {
            destination,
            title,
            children,
        };

        Ok((input, link))
    }
}

````

`src/parser/inline/mod.rs`:

````
mod autolink;
mod code_span;
mod emphasis;
mod footnote_reference;
mod hard_newline;
mod html_entity;
mod image;
mod inline_link;
mod reference_link;
mod strikethrough;
mod text;

#[cfg(test)]
mod tests;

use crate::ast::Inline;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    combinator::{fail, map},
    multi::{many0, many1},
    IResult, Parser,
};
use std::rc::Rc;

use super::util::conditional_inline;

pub(crate) fn inline_many0<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Inline>> {
    move |input: &'a str| {
        let (input, list_of_lists) = many0(inline(state.clone())).parse(input)?;
        let r: Vec<_> = list_of_lists.into_iter().flatten().collect();
        Ok((input, r))
    }
}

pub(crate) fn inline_many1<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Inline>> {
    move |input: &'a str| {
        let (input, list_of_lists) = many1(inline(state.clone())).parse(input)?;
        let r: Vec<_> = list_of_lists.into_iter().flatten().collect();
        Ok((input, r))
    }
}

pub(crate) fn inline<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Inline>> {
    move |input: &'a str| {
        alt((
            conditional_inline(
                state.config.inline_autolink_behavior.clone(),
                map(crate::parser::inline::autolink::autolink, Inline::Autolink),
            ),
            conditional_inline(
                state.config.inline_link_behavior.clone(),
                map(
                    crate::parser::inline::inline_link::inline_link(state.clone()),
                    Inline::Link,
                ),
            ),
            conditional_inline(
                state.config.inline_footnote_reference_behavior.clone(),
                crate::parser::inline::footnote_reference::footnote_reference,
            ),
            conditional_inline(
                state.config.inline_reference_link_behavior.clone(),
                crate::parser::inline::reference_link::reference_link(state.clone()),
            ),
            conditional_inline(
                state.config.inline_hard_newline_behavior.clone(),
                crate::parser::inline::hard_newline::hard_newline,
            ),
            conditional_inline(
                state.config.inline_image_behavior.clone(),
                crate::parser::inline::image::image(state.clone()),
            ),
            conditional_inline(
                state.config.inline_code_span_behavior.clone(),
                map(crate::parser::inline::code_span::code_span, Inline::Code),
            ),
            conditional_inline(
                state.config.inline_emphasis_behavior.clone(),
                crate::parser::inline::emphasis::emphasis(state.clone()),
            ),
            conditional_inline(
                state.config.inline_strikethrough_behavior.clone(),
                crate::parser::inline::strikethrough::strikethrough(state.clone()),
            ),
            custom_parser(state.clone()),
            conditional_inline(
                state.config.inline_text_behavior.clone(),
                crate::parser::inline::text::text(state.clone()),
            ),
        ))
        .parse(input)
    }
}

fn custom_parser(state: Rc<MarkdownParserState>) -> impl FnMut(&str) -> IResult<&str, Vec<Inline>> {
    move |input: &str| {
        if let Some(custom_parser) = state.config.custom_inline_parser.as_ref() {
            let mut p = (**custom_parser).borrow_mut();
            (p.as_mut())(input)
        } else {
            fail().parse(input)
        }
    }
}

````

`src/parser/inline/reference_link.rs`:

````
use crate::ast::{Inline, LinkReference};
use crate::parser::link_util::link_label;
use crate::parser::MarkdownParserState;
use nom::{branch::alt, bytes::complete::tag, sequence::terminated, IResult, Parser};
use std::rc::Rc;

pub(crate) fn reference_link<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Inline> {
    move |input: &'a str| {
        alt((
            reference_link_full(state.clone()),
            reference_link_collapsed(state.clone()),
            reference_link_shortcut(state.clone()),
        ))
        .parse(input)
    }
}

pub(crate) fn reference_link_full<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Inline> {
    move |input: &'a str| {
        let (input, (text, label)) =
            (link_label(state.clone()), link_label(state.clone())).parse(input)?;
        let link_reference = LinkReference { label, text };
        Ok((input, Inline::LinkReference(link_reference)))
    }
}

pub(crate) fn reference_link_collapsed<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Inline> {
    move |input: &'a str| {
        let (input, text) = terminated(link_label(state.clone()), tag("[]")).parse(input)?;
        let link_reference = LinkReference {
            label: text.clone(),
            text,
        };
        Ok((input, Inline::LinkReference(link_reference)))
    }
}

pub(crate) fn reference_link_shortcut<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Inline> {
    move |input: &'a str| {
        let (input, text) = link_label(state.clone()).parse(input)?;
        let link_reference = LinkReference {
            label: text.clone(),
            text,
        };
        Ok((input, Inline::LinkReference(link_reference)))
    }
}

````

`src/parser/inline/strikethrough.rs`:

````
use crate::ast::Inline;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char},
    combinator::{not, peek, recognize, value},
    multi::many1,
    sequence::{preceded, terminated},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn strikethrough<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Inline> {
    move |input: &'a str| {
        let (input, _) = terminated(tag("~~"), peek(not(char('~')))).parse(input)?;
        let not_a_closing_tag = (tag("~~"), char('~'));
        let closing_tag = preceded(peek(not(not_a_closing_tag)), tag("~~"));
        let content_parser = recognize(many1(preceded(
            peek(not(closing_tag)),
            alt((value('~', tag("\\~")), anychar)),
        )));
        let (input, content) = recognize(content_parser).parse(input)?;
        let (input, _) = tag("~~").parse(input)?;

        let (_, inline) = crate::parser::inline::inline_many1(state.clone()).parse(content)?;

        Ok((input, Inline::Strikethrough(inline)))
    }
}

````

`src/parser/inline/tests/autolink.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn autolink1() {
    let doc = parse_markdown(MarkdownParserState::default(), "<http://foo.bar.baz>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Autolink(
                "http://foo.bar.baz".to_owned()
            )])]
        }
    );
}

#[test]
fn autolink2() {
    let doc = parse_markdown(MarkdownParserState::default(), "<irc://foo.bar:2233/baz>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Autolink(
                "irc://foo.bar:2233/baz".to_owned()
            )])]
        }
    );
}

#[test]
fn autolink3() {
    let doc = parse_markdown(MarkdownParserState::default(), "<MAILTO:FOO@BAR.BAZ>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Autolink(
                "MAILTO:FOO@BAR.BAZ".to_owned()
            )])]
        }
    );
}

#[test]
fn autolink4() {
    let doc = parse_markdown(MarkdownParserState::default(), "<http://foo.bar/baz bim>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(
                "<http://foo.bar/baz bim>".to_owned()
            )])]
        }
    );
}

#[test]
fn autolink5() {
    let doc = parse_markdown(MarkdownParserState::default(), "<http://example.com/\\[\\>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Autolink(
                "http://example.com/\\[\\".to_owned()
            )])]
        }
    );
}

#[test]
fn autolink6() {
    let doc = parse_markdown(MarkdownParserState::default(), "<>").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("<>".to_owned())])]
        }
    );
}

````

`src/parser/inline/tests/code_span.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn code_span1() {
    let doc = parse_markdown(MarkdownParserState::default(), "`foo`").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Code("foo".to_string())])],
        }
    );
}

#[test]
fn code_span2() {
    let doc = parse_markdown(MarkdownParserState::default(), "`` foo ` bar ``").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Code(
                "foo ` bar".to_string()
            )])],
        }
    );
}

#[test]
fn code_span3() {
    let doc = parse_markdown(
        MarkdownParserState::default(),
        "``
foo
bar  
baz
``",
    )
    .unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Code(
                "foo bar   baz".to_string()
            )])],
        }
    );
}

````

`src/parser/inline/tests/emphasis.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn emphasis1() {
    let doc = parse_markdown(MarkdownParserState::default(), "*foo bar*").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Emphasis(vec![
                Inline::Text("foo bar".to_string())
            ])])],
        }
    );
}

#[test]
fn emphasis2() {
    let doc = parse_markdown(MarkdownParserState::default(), "* a *").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::List(List {
                kind: ListKind::Bullet(ListBulletKind::Star),
                items: vec![ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph(vec![Inline::Text("a *".to_owned())])]
                }]
            })]
        }
    );
}

#[test]
fn emphasis3() {
    let doc = parse_markdown(MarkdownParserState::default(), "foo ___bar___").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![
                Inline::Text("foo ".to_owned()),
                Inline::Strong(vec![Inline::Emphasis(vec![Inline::Text("bar".to_owned())])])
            ])]
        }
    );
}

#[test]
fn emphasis4() {
    let doc = parse_markdown(MarkdownParserState::default(), "**foo ___bar___ baz**").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Strong(vec![
                Inline::Text("foo ".to_owned()),
                Inline::Strong(vec![Inline::Emphasis(vec![Inline::Text("bar".to_owned())])]),
                Inline::Text(" baz".to_owned())
            ])])]
        }
    );
}

````

`src/parser/inline/tests/footnote_reference.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn footnote_reference1() {
    let doc = parse_markdown(MarkdownParserState::default(), "[^label]").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::FootnoteReference(
                "label".to_string()
            )])],
        }
    );
}

````

`src/parser/inline/tests/hard_newline.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn hard_newline1() {
    let doc = parse_markdown(MarkdownParserState::default(), "line1\\\nline2").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![
                Inline::Text("line1".to_string()),
                Inline::LineBreak,
                Inline::Text("line2".to_string())
            ])],
        }
    );
}

#[test]
fn hard_newline2() {
    let doc = parse_markdown(MarkdownParserState::default(), "line1  \nline2").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![
                Inline::Text("line1".to_string()),
                Inline::LineBreak,
                Inline::Text("line2".to_string())
            ])],
        }
    );
}

````

`src/parser/inline/tests/html_entity.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn html_entity1() {
    let doc = parse_markdown(MarkdownParserState::default(), "&amp;").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("&".to_string())]),]
        }
    );
}

#[test]
fn html_entity2() {
    let doc = parse_markdown(MarkdownParserState::default(), "&#32;").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(" ".to_string())]),]
        }
    );
}

#[test]
fn html_entity3() {
    let doc = parse_markdown(MarkdownParserState::default(), "&#x20;").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(" ".to_string())]),]
        }
    );
}

#[test]
fn html_entity4() {
    let doc = parse_markdown(MarkdownParserState::default(), "&unknownchar;").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(
                "&unknownchar;".to_string()
            )]),]
        }
    );
}

````

`src/parser/inline/tests/image.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn image1() {
    let doc = parse_markdown(MarkdownParserState::default(), "![foo](/url \"title\")").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Image(Image {
                destination: "/url".to_owned(),
                title: Some("title".to_owned()),
                alt: "foo".to_owned(),
            })])]
        }
    );
}

#[test]
fn image2() {
    let doc = parse_markdown(MarkdownParserState::default(), "![foo](train.jpg)").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Image(Image {
                destination: "train.jpg".to_owned(),
                title: None,
                alt: "foo".to_owned(),
            })])]
        }
    );
}

#[test]
fn image3() {
    let doc = parse_markdown(MarkdownParserState::default(), "![foo](<url>)").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Image(Image {
                destination: "url".to_owned(),
                title: None,
                alt: "foo".to_owned(),
            })])]
        }
    );
}

````

`src/parser/inline/tests/inline_link.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn inline_link1() {
    let doc = parse_markdown(MarkdownParserState::default(), "[foo](/url \"title\")").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Link(Link {
                destination: "/url".to_owned(),
                title: Some("title".to_owned()),
                children: vec![Inline::Text("foo".to_owned())]
            })])]
        }
    );
}

#[test]
fn inline_link2() {
    let doc = parse_markdown(MarkdownParserState::default(), "[foo](train.jpg)").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Link(Link {
                destination: "train.jpg".to_owned(),
                title: None,
                children: vec![Inline::Text("foo".to_owned())]
            })])]
        }
    );
}

#[test]
fn inline_link3() {
    let doc = parse_markdown(MarkdownParserState::default(), "[foo](<url>)").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Link(Link {
                destination: "url".to_owned(),
                title: None,
                children: vec![Inline::Text("foo".to_owned())]
            })])]
        }
    );
}

````

`src/parser/inline/tests/mod.rs`:

````
mod autolink;
mod code_span;
mod emphasis;
mod footnote_reference;
mod hard_newline;
mod html_entity;
mod image;
mod inline_link;
mod reference_link;
mod strikethrough;

````

`src/parser/inline/tests/reference_link.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn reference_link1() {
    let doc = parse_markdown(MarkdownParserState::default(), "[text][label]").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::LinkReference(
                LinkReference {
                    label: vec![Inline::Text("label".to_owned())],
                    text: vec![Inline::Text("text".to_owned())],
                }
            )])],
        }
    );
}

#[test]
fn reference_link2() {
    let doc = parse_markdown(MarkdownParserState::default(), "[text][]").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::LinkReference(
                LinkReference {
                    label: vec![Inline::Text("text".to_owned())],
                    text: vec![Inline::Text("text".to_owned())]
                }
            )])],
        }
    );
}

#[test]
fn reference_link3() {
    let doc = parse_markdown(MarkdownParserState::default(), "[text]").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::LinkReference(
                LinkReference {
                    label: vec![Inline::Text("text".to_owned())],
                    text: vec![Inline::Text("text".to_owned())]
                }
            )])],
        }
    );
}

````

`src/parser/inline/tests/strikethrough.rs`:

````
use crate::ast::*;
use crate::parser::{parse_markdown, MarkdownParserState};

#[test]
fn strikethrough1() {
    let doc = parse_markdown(MarkdownParserState::default(), "~~text~~").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Strikethrough(vec![
                Inline::Text("text".to_string())
            ])])],
        }
    );
}

#[test]
fn strikethrough2() {
    let doc = parse_markdown(MarkdownParserState::default(), "~~text~~~").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![Inline::Strikethrough(vec![
                Inline::Text("text~".to_string())
            ])])],
        }
    );
}

#[test]
fn strikethrough3() {
    let doc = parse_markdown(MarkdownParserState::default(), "~~~text~~~").unwrap();
    assert_eq!(
        doc,
        Document {
            blocks: vec![Block::Paragraph(vec![
                Inline::Text("~".to_string()),
                Inline::Strikethrough(vec![Inline::Text("text~".to_string())])
            ])],
        }
    );
}

````

`src/parser/inline/text.rs`:

````
use crate::parser::MarkdownParserState;
use crate::{ast::Inline, parser::util::conditional_inline_unit};
use nom::{
    branch::alt,
    character::complete::{anychar, char, one_of},
    combinator::{map, not, peek, recognize, value},
    multi::many1,
    sequence::preceded,
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn text<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Inline> {
    move |input: &'a str| {
        map(
            many1(alt((
                map(escaped_char, |c| c.to_string()),
                map(
                    crate::parser::inline::html_entity::html_entity(state.clone()),
                    |c| c.to_string(),
                ),
                map(
                    recognize(many1(preceded(peek(is_text(state.clone())), anychar))),
                    |c| c.to_string(),
                ),
            ))),
            |vec| Inline::Text(vec.join("")),
        )
        .parse(input)
    }
}

fn is_text<'a>(state: Rc<MarkdownParserState>) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |input: &'a str| not(not_a_text(state.clone())).parse(input)
}

fn not_a_text<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<()>> {
    move |input: &'a str| {
        alt((
            conditional_inline_unit(
                state.config.inline_autolink_behavior.clone(),
                value((), crate::parser::inline::autolink::autolink),
            ),
            conditional_inline_unit(
                state.config.inline_reference_link_behavior.clone(),
                value(
                    (),
                    crate::parser::inline::reference_link::reference_link(state.clone()),
                ),
            ),
            conditional_inline_unit(
                state.config.inline_hard_newline_behavior.clone(),
                value((), crate::parser::inline::hard_newline::hard_newline),
            ),
            conditional_inline_unit(
                state.config.inline_text_behavior.clone(),
                value(
                    (),
                    crate::parser::inline::html_entity::html_entity(state.clone()),
                ),
            ),
            conditional_inline_unit(
                state.config.inline_image_behavior.clone(),
                value((), crate::parser::inline::image::image(state.clone())),
            ),
            conditional_inline_unit(
                state.config.inline_link_behavior.clone(),
                value(
                    (),
                    crate::parser::inline::inline_link::inline_link(state.clone()),
                ),
            ),
            conditional_inline_unit(
                state.config.inline_code_span_behavior.clone(),
                value((), crate::parser::inline::code_span::code_span),
            ),
            conditional_inline_unit(
                state.config.inline_emphasis_behavior.clone(),
                value((), crate::parser::inline::emphasis::emphasis(state.clone())),
            ),
            conditional_inline_unit(
                state.config.inline_footnote_reference_behavior.clone(),
                value(
                    (),
                    crate::parser::inline::footnote_reference::footnote_reference,
                ),
            ),
            conditional_inline_unit(
                state.config.inline_strikethrough_behavior.clone(),
                value(
                    (),
                    crate::parser::inline::strikethrough::strikethrough(state.clone()),
                ),
            ),
        ))
        .parse(input)
    }
}

fn escaped_char(input: &str) -> IResult<&str, char> {
    preceded(char('\\'), one_of("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~")).parse(input)
}

````

`src/parser/link_util.rs`:

````
use nom::character::complete::{anychar, char, none_of, one_of, satisfy};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, not, peek, recognize, value, verify},
    multi::{fold_many0, many0, many1},
    sequence::{delimited, preceded},
    IResult, Parser,
};
use std::rc::Rc;

use super::MarkdownParserState;

pub(crate) fn link_label<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<crate::ast::Inline>> {
    move |input: &'a str| {
        delimited(tag("["), link_label_inner(state.clone()), tag("]")).parse(input)
    }
}

fn link_label_inner<'a>(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<crate::ast::Inline>> {
    move |input: &'a str| {
        let (input, label_chars) = verify(
            many1(preceded(
                peek(not(char(']'))),
                alt((value(']', tag("\\]")), anychar)),
            )),
            |chars: &[char]| chars.iter().any(|&c| c != ' ' && c != '\n') && chars.len() < 1000,
        )
        .parse(input)?;

        let label = label_chars.iter().collect::<String>();

        let (_, label) = crate::parser::inline::inline_many1(state.clone())
            .parse(label.as_str())
            .map_err(|err| err.map_input(|_| input))?;

        Ok((input, label))
    }
}

pub(crate) fn link_title(input: &str) -> IResult<&str, String> {
    alt((
        link_title_double_quoted,
        link_title_single_quoted,
        link_title_parenthesized,
    ))
    .parse(input)
}

fn link_title_parenthesized(input: &str) -> IResult<&str, String> {
    delimited(char('('), link_title_inner(')'), char(')')).parse(input)
}

fn link_title_single_quoted(input: &str) -> IResult<&str, String> {
    delimited(char('\''), link_title_inner('\''), char('\'')).parse(input)
}

fn link_title_double_quoted(input: &str) -> IResult<&str, String> {
    delimited(tag("\""), link_title_inner('"'), tag("\"")).parse(input)
}

fn link_title_inner(end_delim: char) -> impl FnMut(&str) -> IResult<&str, String> {
    move |input: &str| {
        fold_many0(
            alt((
                map(escaped_char, |c| c.to_string()),
                map(none_of(&[end_delim, '\\'][..]), |c| c.to_string()),
            )),
            String::new,
            |mut acc, s| {
                acc.push_str(&s);
                acc
            },
        )
        .parse(input)
    }
}

fn escaped_char(input: &str) -> IResult<&str, char> {
    preceded(tag("\\"), anychar).parse(input)
}

pub(crate) fn link_destination(input: &str) -> IResult<&str, String> {
    alt((link_destination1, link_destination2)).parse(input)
}

fn link_destination1(input: &str) -> IResult<&str, String> {
    let (input, _) = char('<').parse(input)?;

    let (input, chars) = many0(alt((
        preceded(char('\\'), one_of("<>")),
        preceded(peek(not(one_of("\n<>"))), anychar),
    )))
    .parse(input)?;
    let (input, _) = char('>').parse(input)?;

    let v: String = chars.iter().collect();

    Ok((input, v))
}

fn link_destination2(input: &str) -> IResult<&str, String> {
    let (input, _) = peek(satisfy(|c| is_valid_char(c) && c != '<')).parse(input)?;

    map(
        recognize(many1(alt((
            value((), escaped_char),
            value((), balanced_parens),
            value((), satisfy(|c| is_valid_char(c) && c != '(' && c != ')')),
        )))),
        |s: &str| s.to_string(),
    )
    .parse(input)
}

fn balanced_parens(input: &str) -> IResult<&str, String> {
    delimited(
        tag("("),
        map(
            fold_many0(
                alt((
                    map(escaped_char, |c| c.to_string()),
                    map(balanced_parens, |s| format!("({})", s)),
                    map(satisfy(|c| is_valid_char(c) && c != '(' && c != ')'), |c| {
                        c.to_string()
                    }),
                )),
                String::new,
                |mut acc, item| {
                    acc.push_str(&item);
                    acc
                },
            ),
            |s| s,
        ),
        tag(")"),
    )
    .parse(input)
}

fn is_valid_char(c: char) -> bool {
    !c.is_ascii_control() && c != ' ' && c != '<'
}

````

`src/parser/mod.rs`:

````
mod blocks;
pub mod config;
mod inline;
mod link_util;
mod util;

use crate::ast::Document;
use crate::parser::config::MarkdownParserConfig;
use nom::{
    branch::alt,
    character::complete::{line_ending, space1},
    combinator::eof,
    multi::many0,
    sequence::terminated,
    Parser,
};
use std::rc::Rc;

pub struct MarkdownParserState {
    pub config: Rc<MarkdownParserConfig>,
}

impl MarkdownParserState {
    pub fn with_config(config: MarkdownParserConfig) -> Self {
        Self {
            config: Rc::new(config),
        }
    }
}

impl Default for MarkdownParserState {
    fn default() -> Self {
        Self::with_config(MarkdownParserConfig::default())
    }
}

/// Parse the given Markdown string into an AST.
pub fn parse_markdown(
    state: MarkdownParserState,
    input: &str,
) -> Result<Document, nom::Err<nom::error::Error<&str>>> {
    let empty_lines = many0(alt((space1, line_ending)));
    let mut parser = terminated(
        many0(crate::parser::blocks::block(Rc::new(state))),
        (empty_lines, eof),
    );
    let (_, blocks) = parser.parse(input)?;

    let blocks = blocks.into_iter().flatten().collect();

    Ok(Document { blocks })
}

````

`src/parser/util.rs`:

````
use crate::ast::{Block, Inline};
use nom::{
    branch::alt,
    character::complete::{anychar, line_ending, not_line_ending, space0},
    combinator::{eof, fail, not, recognize, value},
    multi::{many0, many1},
    sequence::{preceded, terminated},
    IResult, Parser,
};

pub(crate) fn eof_or_eol(input: &str) -> IResult<&str, &str> {
    alt((line_ending, eof)).parse(input)
}

pub(crate) fn many_empty_lines0(input: &str) -> IResult<&str, Vec<&str>> {
    many0(preceded(space0, eof_or_eol)).parse(input)
}

pub(crate) fn not_eof_or_eol1(input: &str) -> IResult<&str, &str> {
    recognize(many1(preceded(not(eof_or_eol), anychar))).parse(input)
}

pub(crate) fn not_eof_or_eol0(input: &str) -> IResult<&str, &str> {
    alt((not_line_ending, eof)).parse(input)
}

pub(crate) fn line_terminated<'a, O, P>(
    inner: P,
) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    terminated(inner, eof_or_eol)
}

// pub(crate) fn logged<'a, O, P>(
//     message: &'static str,
//     mut inner: P,
// ) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
// where
//     P: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
//     O: std::fmt::Debug,
// {
//     move |input: &'a str| {
//         println!("Logged: {message}: {:?}", input);
//         let r = inner.parse(input);
//         println!("Logged out: {message}: {:?}", r);
//         r
//     }
// }

pub(crate) fn conditional<'a, O, P>(
    behavior: crate::parser::config::ElementBehavior<O>,
    default: Vec<O>,
    mut inner: P,
) -> impl Parser<&'a str, Output = Vec<O>, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
    O: Clone,
{
    move |input: &'a str| {
        let inner1 = |s: &'a str| inner.parse(s);
        match &behavior {
            crate::parser::config::ElementBehavior::Ignore => fail().map(|v| vec![v]).parse(input),
            crate::parser::config::ElementBehavior::Parse => inner1.map(|v| vec![v]).parse(input),
            crate::parser::config::ElementBehavior::Skip => {
                value(default.clone(), inner1.map(|v| vec![v])).parse(input)
            }
            crate::parser::config::ElementBehavior::Map(f) => {
                let (i, o) = inner.parse(input)?;
                let mut f1 = (**f).borrow_mut();
                let mapped = vec![(f1.as_mut())(o)];
                Ok((i, mapped))
            }
            crate::parser::config::ElementBehavior::FlatMap(f) => {
                let (i, o) = inner.parse(input)?;
                let mut f1 = (**f).borrow_mut();
                let mapped = (f1.as_mut())(o);
                Ok((i, mapped))
            }
        }
    }
}

pub(crate) fn conditional_block_unit<'a, P>(
    behavior: crate::parser::config::ElementBehavior<Block>,
    mut inner: P,
) -> impl Parser<&'a str, Output = Vec<()>, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = (), Error = nom::error::Error<&'a str>>,
{
    let behavior: crate::parser::config::ElementBehavior<()> = match behavior {
        super::config::ElementBehavior::Parse => super::config::ElementBehavior::Parse,
        super::config::ElementBehavior::Ignore => super::config::ElementBehavior::Ignore,
        super::config::ElementBehavior::Skip => super::config::ElementBehavior::Skip,
        super::config::ElementBehavior::Map(_) => super::config::ElementBehavior::Parse,
        super::config::ElementBehavior::FlatMap(_) => super::config::ElementBehavior::Parse,
    };
    move |input: &'a str| {
        let inner1 = |s: &'a str| inner.parse(s);
        conditional(behavior.clone(), vec![()], inner1).parse(input)
    }
}

pub(crate) fn conditional_inline_unit<'a, P>(
    behavior: crate::parser::config::ElementBehavior<Inline>,
    mut inner: P,
) -> impl Parser<&'a str, Output = Vec<()>, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = (), Error = nom::error::Error<&'a str>>,
{
    let behavior: crate::parser::config::ElementBehavior<()> = match behavior {
        super::config::ElementBehavior::Parse => super::config::ElementBehavior::Parse,
        super::config::ElementBehavior::Ignore => super::config::ElementBehavior::Ignore,
        super::config::ElementBehavior::Skip => super::config::ElementBehavior::Skip,
        super::config::ElementBehavior::Map(_) => super::config::ElementBehavior::Parse,
        super::config::ElementBehavior::FlatMap(_) => super::config::ElementBehavior::Parse,
    };
    move |input: &'a str| {
        let inner1 = |s: &'a str| inner.parse(s);
        conditional(behavior.clone(), vec![()], inner1).parse(input)
    }
}

pub(crate) fn conditional_block<'a, P>(
    behavior: crate::parser::config::ElementBehavior<Block>,
    mut inner: P,
) -> impl Parser<&'a str, Output = Vec<Block>, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = Block, Error = nom::error::Error<&'a str>>,
{
    move |input: &'a str| {
        let inner1 = |s: &'a str| inner.parse(s);
        conditional(behavior.clone(), vec![Block::Empty], inner1).parse(input)
    }
}

pub(crate) fn conditional_inline<'a, P>(
    behavior: crate::parser::config::ElementBehavior<Inline>,
    mut inner: P,
) -> impl Parser<&'a str, Output = Vec<Inline>, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = Inline, Error = nom::error::Error<&'a str>>,
{
    move |input: &'a str| {
        let inner1 = |s: &'a str| inner.parse(s);
        conditional(behavior.clone(), vec![Inline::Empty], inner1).parse(input)
    }
}

````

`src/printer/block.rs`:

````
use crate::ast::*;
use crate::printer::{inline::ToDocInline, ToDoc};
use pretty::{Arena, DocAllocator, DocBuilder};
use std::rc::Rc;

impl<'a> ToDoc<'a> for Vec<Block> {
    fn to_doc(
        &self,
        config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        let refs: Vec<_> = self.iter().collect();
        refs.to_doc(config, arena)
    }
}

impl<'a> ToDoc<'a> for Vec<&Block> {
    fn to_doc(
        &self,
        config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        let mut acc = arena.nil();
        for (i, block) in self.iter().enumerate() {
            if i > 0 {
                // first block should not have an empty line before it
                acc = acc.append(arena.hardline());
                if matches!(block, Block::List(_)) {
                    if config.empty_line_before_list {
                        // empty line before list block
                        acc = acc.append(arena.hardline());
                    }
                } else {
                    acc = acc.append(arena.hardline());
                }
            }
            acc = acc.append(block.to_doc(config.clone(), arena))
        }
        acc
    }
}

/// Block-level nodes
impl<'a> ToDoc<'a> for Block {
    fn to_doc(
        &self,
        config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        match self {
            Block::Paragraph(inlines) => inlines.to_doc_inline(true, arena),
            Block::Heading(v) => v.to_doc(config, arena),
            Block::ThematicBreak => arena.text("---"),
            Block::BlockQuote(inner) => {
                crate::printer::blockquote::blockquote_to_doc(config, arena, inner)
            }
            Block::List(v) => v.to_doc(config, arena),
            Block::CodeBlock(CodeBlock { kind, literal }) => {
                match kind {
                    CodeBlockKind::Fenced { info } => {
                        let info = info.as_deref().unwrap_or("");
                        arena
                            .text(format!("```{}\n", info))
                            .append(arena.text(literal.clone()))
                            .append(arena.text("\n```"))
                    }
                    CodeBlockKind::Indented => {
                        // каждый строка с отступом 4 пробела
                        let indented = literal
                            .lines()
                            .map(|l| format!("    {}", l))
                            .collect::<Vec<_>>()
                            .join("\n");
                        arena.text(indented)
                    }
                }
            }
            Block::HtmlBlock(html) => arena.text(html.clone()),
            Block::Definition(def) => arena
                .text("[")
                .append(def.label.to_doc_inline(true, arena))
                .append(arena.text("]: "))
                .append(arena.text(format!(
                    "{}{}",
                    def.destination,
                    def.title
                        .as_ref()
                        .map(|t| format!(" \"{}\"", t))
                        .unwrap_or_default()
                ))),

            Block::Empty => arena.nil(),
            Block::Table(v) => v.to_doc(config, arena),
            Block::FootnoteDefinition(def) => arena
                .text(format!("[^{}]: ", def.label))
                .append(def.blocks.to_doc(config, arena)),
        }
    }
}

````

`src/printer/blockquote.rs`:

````
use crate::ast::*;
use crate::printer::ToDoc;
use pretty::{Arena, DocAllocator, DocBuilder};
use std::rc::Rc;

pub(crate) fn blockquote_to_doc<'a>(
    config: Rc<crate::printer::config::Config>,
    arena: &'a Arena<'a>,
    inner: &[Block],
) -> DocBuilder<'a, Arena<'a>, ()> {
    let blocks = inner.to_owned();
    arena.column(move |current_column| {
        let prefix = "> ";
        let tmp_arena = Arena::new();
        let doc = blocks.to_doc(config.clone(), &tmp_arena);

        let mut buf = Vec::new();
        doc.render(config.width - current_column - prefix.len(), &mut buf)
            .unwrap();
        let text = String::from_utf8(buf).unwrap();

        let lines = text.lines().map(|d| {
            arena
                .as_string(prefix.to_string())
                .append(arena.as_string(d))
        });

        arena.intersperse(lines, arena.hardline()).into_doc()
    })
}

````

`src/printer/config.rs`:

````
pub struct Config {
    pub(crate) width: usize,
    pub(crate) spaces_before_list_item: usize,
    pub(crate) empty_line_before_list: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 80,
            spaces_before_list_item: 1,
            empty_line_before_list: true,
        }
    }
}

impl Config {
    /// Render document with a given width in characters.
    pub fn with_width(self, width: usize) -> Self {
        Self { width, ..self }
    }

    /// Sets the number of spaces to insert before each list item when rendering the AST to Markdown.
    ///
    /// The default is 1 space. According to the GitHub Flavored Markdown specification,
    /// between 0 and 3 spaces before a list marker are allowed:
    /// <https://github.github.com/gfm/#lists>
    ///
    /// # Parameters
    ///
    /// - `spaces`: the number of spaces to insert before each list item (valid range: 0..=3).
    ///
    /// # Returns
    ///
    /// A new printer config instance with `spaces_before_list_item` set to the specified value.
    pub fn with_spaces_before_list_item(self, spaces: usize) -> Self {
        Self {
            spaces_before_list_item: spaces,
            ..self
        }
    }

    /// Sets whether to add an empty line before lists.
    ///
    /// The default is `true`, which means that lists are preceded by an empty line.
    pub fn with_empty_line_before_list(self, tight: bool) -> Self {
        Self {
            empty_line_before_list: tight,
            ..self
        }
    }
}

````

`src/printer/heading.rs`:

````
use crate::ast::*;
use crate::printer::{inline::ToDocInline, ToDoc};
use pretty::{Arena, DocAllocator, DocBuilder};
use std::rc::Rc;

impl<'a> ToDoc<'a> for Heading {
    fn to_doc(
        &self,
        _config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        match self.kind {
            HeadingKind::Atx(level) => {
                let hashes = "#".repeat(level as usize);
                arena
                    .text(hashes)
                    .append(arena.space())
                    .append(self.content.to_doc_inline(false, arena))
            }
            HeadingKind::Setext(SetextHeading::Level1) => self
                .content
                .to_doc_inline(true, arena)
                .append(arena.hardline())
                .append(arena.text("==========")),
            HeadingKind::Setext(SetextHeading::Level2) => self
                .content
                .to_doc_inline(true, arena)
                .append(arena.hardline())
                .append(arena.text("----------")),
        }
    }
}

````

`src/printer/inline.rs`:

````
use crate::ast::*;
use pretty::{Arena, DocAllocator, DocBuilder};

pub(crate) trait ToDocInline<'a> {
    fn to_doc_inline(
        &self,
        allow_newlines: bool,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()>;
}

impl<'a> ToDocInline<'a> for Vec<Inline> {
    fn to_doc_inline(
        &self,
        allow_newlines: bool,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        arena.concat(
            self.iter()
                .map(|inline| inline.to_doc_inline(allow_newlines, arena))
                .collect::<Vec<_>>(),
        )
    }
}

impl<'a> ToDocInline<'a> for Inline {
    fn to_doc_inline(
        &self,
        allow_newlines: bool,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        match self {
            Inline::Text(t) => {
                let words_or_spaces: Vec<_> = split_with_spaces(t);
                let separator = if allow_newlines {
                    arena.softline()
                } else {
                    arena.space()
                };
                let words_or_spaces = words_or_spaces.into_iter().map(|v| match v {
                    Some(v) => arena.text(v.to_string()),
                    None => separator.clone(),
                });
                arena.concat(words_or_spaces)
            }
            // TODO parametrize format
            Inline::LineBreak => arena.text("  \n"),
            Inline::Code(code) => arena.text("`").append(code.clone()).append(arena.text("`")),
            Inline::Html(html) => arena.text(html.clone()),
            Inline::Emphasis(children) => arena
                .text("*")
                .append(children.to_doc_inline(allow_newlines, arena))
                .append(arena.text("*")),
            Inline::Strong(children) => arena
                .text("**")
                .append(children.to_doc_inline(allow_newlines, arena))
                .append(arena.text("**")),
            Inline::Strikethrough(children) => arena
                .text("~~")
                .append(children.to_doc_inline(allow_newlines, arena))
                .append(arena.text("~~")),
            Inline::Link(Link {
                destination,
                title,
                children,
            }) => {
                let title = match title {
                    Some(v) => arena
                        .text(" \"")
                        .append(arena.text(v.clone()))
                        .append(arena.text("\"")),
                    None => arena.nil(),
                };
                arena
                    .text("[")
                    .append(children.to_doc_inline(allow_newlines, arena))
                    .append(arena.text("]("))
                    .append(arena.text(destination.clone()))
                    .append(title)
                    .append(")")
            }
            Inline::Image(Image {
                destination,
                title,
                alt,
            }) => {
                let title_part = title
                    .as_ref()
                    .map(|t| format!(" \"{}\"", t))
                    .unwrap_or_default();
                arena
                    .text("![")
                    .append(arena.text(alt.clone()))
                    .append("](")
                    .append(arena.text(destination.clone()))
                    .append(arena.text(title_part))
                    .append(arena.text(")"))
            }
            Inline::Autolink(link) => arena.text(format!("<{}>", link)),
            Inline::FootnoteReference(label) => arena.text(format!("[^{}]", label)),
            Inline::Empty => arena.nil(),
            Inline::LinkReference(v) => {
                if v.label == v.text {
                    return arena
                        .text("[")
                        .append(v.label.to_doc_inline(allow_newlines, arena))
                        .append("]");
                }
                arena
                    .text("[")
                    .append(v.text.to_doc_inline(allow_newlines, arena))
                    .append("][")
                    .append(v.label.to_doc_inline(allow_newlines, arena))
                    .append(arena.text("]"))
            }
        }
    }
}

/// Split string by spaces, but keep the spaces in the result.
fn split_with_spaces(s: &str) -> Vec<Option<&str>> {
    let mut result = Vec::new();
    let mut word_start: Option<usize> = None;

    for (i, c) in s.char_indices() {
        if c.is_whitespace() {
            if let Some(start) = word_start {
                result.push(Some(&s[start..i]));
                word_start = None;
            }
            if result.last().is_none_or(|x| x.is_some()) {
                result.push(None);
            }
        } else if word_start.is_none() {
            word_start = Some(i);
        }
    }

    if let Some(start) = word_start {
        result.push(Some(&s[start..]));
    }

    result
}

````

`src/printer/list.rs`:

````
use crate::ast::*;
use crate::printer::ToDoc;
use pretty::{Arena, DocAllocator, DocBuilder};
use std::rc::Rc;

impl<'a> ToDoc<'a> for List {
    fn to_doc(
        &self,
        config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        let mut counter = if let ListKind::Ordered(v) = &self.kind {
            v.start
        } else {
            0
        };
        let prefix_length = match &self.kind {
            ListKind::Bullet(ListBulletKind::Dash) => 2 + config.spaces_before_list_item, // <space>-<space>
            ListKind::Bullet(ListBulletKind::Star) => 2 + config.spaces_before_list_item, // <space>*<space>
            ListKind::Bullet(ListBulletKind::Plus) => 2 + config.spaces_before_list_item, // <space>+<space>
            ListKind::Ordered(v) => {
                let last = v.start + self.items.len() as u64 - 1;
                let digits = last.to_string().len();
                digits + 2 + config.spaces_before_list_item // <space>1.<space>
            }
        };
        let items = self.items.iter().map(|item| {
            let marker = match self.kind {
                ListKind::Bullet(ListBulletKind::Dash) => "-".to_owned(),
                ListKind::Bullet(ListBulletKind::Star) => "*".to_owned(),
                ListKind::Bullet(ListBulletKind::Plus) => "+".to_owned(),
                ListKind::Ordered(_) => {
                    let r = format!("{}.", counter);
                    counter += 1;
                    r
                }
            };

            let task_list_marker = match item.task {
                Some(TaskState::Complete) => arena.text("[X]").append(arena.space()),
                Some(TaskState::Incomplete) => arena.text("[ ]").append(arena.space()),
                None => arena.nil(),
            };

            let indent = " ".repeat(config.spaces_before_list_item);

            arena
                .text(indent)
                .append(arena.text(marker.clone()))
                .append(arena.space())
                .append(task_list_marker)
                .append(
                    item.blocks
                        .to_doc(config.clone(), arena)
                        .nest(prefix_length as isize)
                        .group(),
                )
        });

        arena.intersperse(items, arena.hardline())
    }
}

````

`src/printer/mod.rs`:

````
mod block;
mod blockquote;
pub mod config;
mod heading;
mod inline;
mod list;
mod table;
mod tests;

use crate::ast::*;
use pretty::{Arena, DocBuilder};
use std::rc::Rc;

/// Render Markdown AST to Markdown document.
pub fn render_markdown(ast: &Document, config: crate::printer::config::Config) -> String {
    let config = Rc::new(config);
    let arena = Arena::new();
    let doc = ast.to_doc(config.clone(), &arena);

    let mut buf = Vec::new();
    doc.render(config.width, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

trait ToDoc<'a> {
    fn to_doc(
        &self,
        config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()>;
}

impl<'a> ToDoc<'a> for Document {
    fn to_doc(
        &self,
        config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        self.blocks.to_doc(config, arena)
    }
}

````

`src/printer/table.rs`:

````
use crate::ast::*;
use crate::printer::{inline::ToDocInline, ToDoc};
use core::iter::Iterator;
use pretty::{Arena, DocAllocator, DocBuilder};
use std::rc::Rc;

impl<'a> ToDoc<'a> for Table {
    fn to_doc(
        &self,
        _config: Rc<crate::printer::config::Config>,
        arena: &'a Arena<'a>,
    ) -> DocBuilder<'a, Arena<'a>, ()> {
        if self.rows.is_empty() {
            return arena.nil();
        }

        let content = table_content(self);
        let columns_width = columns_width(&content, &self.alignments);
        let header = row_to_doc(&content[0], &columns_width, &self.alignments, arena);
        let separator = alignments_row_to_doc(&self.alignments, &columns_width, arena);

        let body = content
            .iter()
            .skip(1)
            .map(|row| row_to_doc(row, &columns_width, &self.alignments, arena))
            .collect::<Vec<_>>();

        let mut rows = vec![header, separator];
        rows.extend(body);

        arena.intersperse(rows, arena.hardline())
    }
}

fn alignments_row_to_doc<'a>(
    alignments: &[Alignment],
    columns_width: &[usize],
    arena: &'a Arena<'a>,
) -> DocBuilder<'a, Arena<'a>, ()> {
    let mut acc = arena.text("| ");
    for (i, alignment) in alignments.iter().enumerate() {
        if i > 0 {
            acc = acc.append(arena.text(" | "))
        }
        let column_width = columns_width.get(i).unwrap_or(&3);
        acc = acc.append(alignment_to_doc(*alignment, *column_width, arena))
    }
    acc.append(arena.text(" |"))
}

fn alignment_to_doc<'a>(
    alignment: Alignment,
    column_width: usize,
    arena: &'a Arena<'a>,
) -> DocBuilder<'a, Arena<'a>, ()> {
    match alignment {
        Alignment::None | Alignment::Left => {
            let repeat = if column_width > 1 { column_width } else { 1 };
            arena.text("-".repeat(repeat))
        }
        Alignment::Center => {
            let repeat = if column_width > 2 {
                column_width - 2
            } else {
                1
            };
            arena
                .text(":")
                .append(arena.text("-".repeat(repeat)))
                .append(arena.text(":"))
        }
        Alignment::Right => {
            let repeat = if column_width > 1 {
                column_width - 1
            } else {
                1
            };
            arena.text("-".repeat(repeat)).append(arena.text(":"))
        }
    }
}

fn row_to_doc<'a>(
    row: &[String],
    columns_width: &[usize],
    alignments: &[Alignment],
    arena: &'a Arena<'a>,
) -> DocBuilder<'a, Arena<'a>, ()> {
    let mut acc = arena.text("| ");
    for (i, cell) in row.iter().enumerate() {
        if i > 0 {
            acc = acc.append(arena.text(" | "))
        }
        let alignment = alignments.get(i).cloned().unwrap_or_default();
        // Unreachable code, because we already checked the length of the row
        let column_width = columns_width.get(i).unwrap_or(&3);
        acc = acc.append(cell_to_doc(cell, *column_width, alignment, arena))
    }
    acc.append(arena.text(" |"))
}

fn cell_to_doc<'a>(
    cell: &str,
    column_width: usize,
    alignment: Alignment,
    arena: &'a Arena<'a>,
) -> DocBuilder<'a, Arena<'a>, ()> {
    let content = match alignment {
        Alignment::None | Alignment::Left => {
            format!(
                "{}{}",
                cell,
                " ".repeat(column_width - cell.chars().count())
            )
        }
        Alignment::Center => {
            let padding = column_width - cell.chars().count();
            let left_padding = padding / 2;
            let right_padding = padding - left_padding;
            format!(
                "{}{}{}",
                " ".repeat(left_padding),
                cell,
                " ".repeat(right_padding)
            )
        }
        Alignment::Right => format!(
            "{}{}",
            " ".repeat(column_width - cell.chars().count()),
            cell
        ),
    };
    arena.text(content)
}

fn columns_width(table: &[Vec<String>], alignments: &[Alignment]) -> Vec<usize> {
    let mut widths = Vec::new();
    for i in 0..table[0].len() {
        let width = column_width(table, alignments, i);
        widths.push(width);
    }
    widths
}

fn column_width(table: &[Vec<String>], alignments: &[Alignment], column_index: usize) -> usize {
    let content_width = column_content_width(table, column_index);
    let alignment_width = match alignments.get(column_index) {
        Some(Alignment::Left) => 1,
        Some(Alignment::Center) => 3,
        Some(Alignment::Right) => 2,
        Some(Alignment::None) => 1,
        None => 1,
    };
    if content_width > alignment_width {
        content_width
    } else {
        alignment_width
    }
}

fn column_content_width(table: &[Vec<String>], column_index: usize) -> usize {
    let mut max_width = 0;
    for row in table {
        if column_index < row.len() {
            let cell_width = row[column_index].chars().count();
            if cell_width > max_width {
                max_width = cell_width;
            }
        }
    }

    max_width
}

fn table_content(table: &Table) -> Vec<Vec<String>> {
    let mut content = Vec::new();
    for row in &table.rows {
        let mut row_content = Vec::new();
        for cell in row {
            let cell_content = render_cell(cell);
            row_content.push(cell_content);
        }
        content.push(row_content);
    }
    content
}

fn render_cell(doc: &Vec<Inline>) -> String {
    let tmp_arena = Arena::new();
    let doc = doc.to_doc_inline(false, &tmp_arena);

    let mut buf = Vec::new();
    doc.render(usize::MAX, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

````

`src/printer/tests/list.rs`:

````
#![cfg(test)]
use rstest::rstest;

#[rstest(
    input,
    case(
        r#"- item1
- item2"#
    ),
    case(
        r#"11. item1
12. item2"#
    ),
    case(
        r#"9. item1
10. item2"#
    ),
    case(
        r#"- item1
- item2
  
  - item2 1
  - item2 2"#
    )
)]
fn symmetric_round_trip_list_with_spaces_before_list_item(input: &str) {
    let config = crate::printer::config::Config::default().with_spaces_before_list_item(0);
    let doc = crate::parser::parse_markdown(crate::parser::MarkdownParserState::default(), input)
        .unwrap();
    println!("{:?} => {:#?}", input, doc);
    let result = crate::printer::render_markdown(&doc, config);
    assert_eq!(input, result);
}

#[rstest(
    input,
    case(
        r#"# head1
 - item1
 - item2

# head2
 - item3
 - item4

# head3
 - item5
 - item6"#
    ),
    case(
        r#" - item1
 - item2
    - item2 1
    - item2 2"#
    ),
    case(
        r#" - item1
 - item2
    - item2 1
    - item2 2
       - item2 2 1
       - item2 2 2"#
    )
)]
fn symmetric_round_trip_list_without_empty_line_before_list(input: &str) {
    let config = crate::printer::config::Config::default().with_empty_line_before_list(false);
    let doc = crate::parser::parse_markdown(crate::parser::MarkdownParserState::default(), input)
        .unwrap();
    println!("{:?} => {:#?}", input, doc);
    let result = crate::printer::render_markdown(&doc, config);
    assert_eq!(input, result);
}

````

`src/printer/tests/mod.rs`:

````
#![cfg(test)]
use rstest::rstest;

mod list;

#[rstest(input,
         case("---"),
        case(
        r#"word1 word2"#),
        case(
        r#"paragraph1

paragraph2"#),
        case(
        r#"# heading1

## heading2

heading 3
=========="#),
        // case( "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."),
         case(r#" 100. item1 paragraph1
      
      item1 paragraph2
 101. item2 paragraph1
      
      item2 paragraph2
      
      item2 paragraph3
 102. item3 paragraph1
      
      item3 paragraph2"#),
        case(
        r#" 1. item 1
    
     * nested list item 1
     * nested list item 2
 2. item 2"#),
        case(
        r#"> line1 line1 line1 line1 line1 line1 line1 line1 line1 line1 line1 line1 line1
> line1 line1 line1 line1 line1 line1 line1 line1"#),
        case(
        r#"> line1 line1 line1 line1 line1 line1 line1 line1 line1 line1 line1 line1
> 
> > line1 line1 line1 line1 line1 line1 line1 line1 line1"#),
        case(
        r#"Это *курсив, но внутри **жирный и *обратно курсив*** снова жирный* конец."#),
        case(
        r#"Это \*не курсив\*, а просто звёздочки."#),
        case(
        r#"Вот [ссылка *с курсивом внутри*](https://example.com) и ещё текст."#),
        case(
        r#"Инлайн код `внутри *курсива*` не должен парситься как курсив."#),
        case(
        r#"Параграф первый с **жирным** текстом.

Параграф второй с *курсивом* и списком:

 - Первый пункт **жирный**
 - Второй пункт *курсивный*

Конец."#),
        case(
            r#"Список с задачами:

 - Первый пункт
 - [ ] Второй пункт
 - [X] Третий пункт

Конец."#),
        case(
        r#"> Список внутри цитаты:

>  - Пункт *первый*
>  - Пункт **второй**
>    
>     - Подпункт `третий`
> 
> Конец цитаты."#),
        case(
        r#"## Это *курсивный* заголовок с [ссылкой](https://example.com)"#),
        case(
        r#"Это просто текст** а потом *ещё* звёздочки."#),
        case(
        r#"[ссылка с `кодом` внутри](https://example.com)"#),
        case(
        r#"Здесь *курсив без конца и **жирный без конца"#),
        case(
        "**Всё жирное и *курсивное и `кодовое внутри курсивного` и снова курсивное* снова\nжирное**"),
        case(
        r#"[Ссылка с \*экранированной звездочкой\* внутри](https://example.com)"#),
        case(
        r#"Текст с сноской[^1].

[^1]: Это текст сноски."#),
        case(
        r#"Это *курсивный текст со сноской[^note]*.

[^note]: Сноска для курсивного текста."#),
        case(
        r#"[Ссылка с сноской[^linknote]](https://example.com)

[^linknote]: Сноска для ссылки."#),
        case(
        r#"# Заголовок со сноской[^headnote]

[^headnote]: Пояснение к заголовку."#),
        case(
        r#"| Заголовок 1 | Заголовок 2 | Заголовок 3 |
| ----------- | ----------: | :---------: |
| Ячейка 1    |    Ячейка 2 |  Ячейка 3   |
| Ячейка 4    |    Ячейка 5 |  Ячейка 6   |"#),
        case(
        r#"| **Заголовок 1** | Заголовок 2 | Заголовок 3 |
| --------------- | ----------: | :---------: |
| Ячейка 1        |    Ячейка 2 |  Ячейка 3   |
| Ячейка 4        |    Ячейка 5 |  Ячейка 6   |"#),
        case(
        r#"> | Заголовок 1 | Заголовок 2 | Заголовок 3 |
> | ----------- | ----------: | :---------: |
> | Ячейка 1    |    Ячейка 2 |  Ячейка 3   |
> | Ячейка 4    |    Ячейка 5 |  Ячейка 6   |"#),
        case(
            r#"> blockquote level 1
> 
> > blockquote level 2"#),
        case(
            r#"text

```rust
let s = "hello\n";

```"#),

        case(
            r#"Autolinks test: <http://example.com> and <johnlepikhin@gmail.com>"#),

)]
fn symmetric_round_trip(input: &str) {
    let config = crate::printer::config::Config::default();
    let doc = crate::parser::parse_markdown(crate::parser::MarkdownParserState::default(), input)
        .unwrap();
    println!("{:?} => {:#?}", input, doc);
    let result = crate::printer::render_markdown(&doc, config);
    assert_eq!(input, result);
}

````
