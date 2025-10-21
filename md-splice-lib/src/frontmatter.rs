use anyhow::{anyhow, Context};
use serde::Deserialize;
use serde_yaml::Value as YamlValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FrontmatterFormat {
    Yaml,
    Toml,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDocument {
    pub frontmatter: Option<YamlValue>,
    pub body: String,
    pub format: Option<FrontmatterFormat>,
    pub(crate) frontmatter_block: Option<String>,
}

impl ParsedDocument {
    fn ensure_format(&mut self) {
        if self.format.is_none() {
            self.format = Some(FrontmatterFormat::Yaml);
        }
    }
}

pub fn parse(content: &str) -> anyhow::Result<ParsedDocument> {
    let mut parsed = ParsedDocument {
        frontmatter: None,
        body: content.to_string(),
        format: None,
        frontmatter_block: None,
    };

    let Some(first_line) = content.lines().next() else {
        return Ok(parsed);
    };

    let first_line = first_line.trim_end_matches('\r');
    let (format, delimiter) = match first_line {
        "---" => (FrontmatterFormat::Yaml, "---"),
        "+++" => (FrontmatterFormat::Toml, "+++"),
        _ => return Ok(parsed),
    };

    let Some(rest) = strip_opening_delimiter(content, delimiter) else {
        return Ok(parsed);
    };

    let (frontmatter_str, body_start_idx) =
        extract_frontmatter_block(rest, delimiter).ok_or_else(|| {
            anyhow!(
                "Failed to locate closing frontmatter delimiter `{delimiter}` at start of document"
            )
        })?;

    let opening_len = content.len() - rest.len();
    let frontmatter_block = &content[..opening_len + body_start_idx];
    let body_slice = &content[opening_len + body_start_idx..];

    let frontmatter_value = match format {
        FrontmatterFormat::Yaml => {
            if frontmatter_str.trim().is_empty() {
                YamlValue::Null
            } else {
                serde_yaml::from_str(frontmatter_str)
                    .with_context(|| "Failed to parse YAML frontmatter at start of document")?
            }
        }
        FrontmatterFormat::Toml => {
            let toml_value: toml::Value = toml::from_str(frontmatter_str)
                .with_context(|| "Failed to parse TOML frontmatter at start of document")?;
            serde_yaml::to_value(toml_value)
                .map_err(|e| anyhow!("Failed to convert TOML frontmatter to YAML value: {e}"))?
        }
    };

    parsed.frontmatter = Some(frontmatter_value);
    parsed.body = body_slice.to_string();
    parsed.format = Some(format);
    parsed.frontmatter_block = Some(frontmatter_block.to_string());

    Ok(parsed)
}

pub fn refresh_frontmatter_block(parsed: &mut ParsedDocument) -> anyhow::Result<()> {
    if parsed.frontmatter.is_some() {
        parsed.ensure_format();
        let format = parsed
            .format
            .ok_or_else(|| anyhow!("Frontmatter format missing during serialization"))?;

        let block = {
            let value = parsed
                .frontmatter
                .as_ref()
                .ok_or_else(|| anyhow!("Frontmatter missing during serialization"))?;
            serialize_frontmatter_block(value, format)?
        };

        parsed.frontmatter_block = Some(block);
    } else {
        parsed.frontmatter_block = None;
        parsed.format = None;
    }

    Ok(())
}

fn serialize_frontmatter_block(
    value: &YamlValue,
    format: FrontmatterFormat,
) -> anyhow::Result<String> {
    let (delimiter, mut serialized) = match format {
        FrontmatterFormat::Yaml => {
            if value.is_null() {
                ("---", String::new())
            } else {
                let yaml = serialize_yaml_value(value)?;
                ("---", yaml)
            }
        }
        FrontmatterFormat::Toml => {
            if value.is_null() {
                ("+++", String::new())
            } else {
                let toml_value: toml::Value =
                    serde_yaml::from_value(value.clone()).map_err(|err| {
                        anyhow!("Failed to convert YAML value into TOML frontmatter: {err}")
                    })?;
                let toml = toml::to_string_pretty(&toml_value)
                    .map_err(|err| anyhow!("Failed to serialize TOML frontmatter: {err}"))?;
                ("+++", toml)
            }
        }
    };

    while serialized.ends_with(['\n', '\r']) {
        serialized.pop();
    }

    let mut block = String::new();
    block.push_str(delimiter);
    block.push('\n');

    if !serialized.is_empty() {
        block.push_str(&serialized);
        block.push('\n');
    }

    block.push_str(delimiter);
    block.push('\n');

    Ok(block)
}

pub fn serialize_yaml_value(value: &YamlValue) -> anyhow::Result<String> {
    let serialized = serde_yaml::to_string(value)?;
    Ok(trim_yaml_document_markers(&serialized))
}

pub fn trim_yaml_document_markers(serialized: &str) -> String {
    let without_start = serialized
        .strip_prefix("---\n")
        .or_else(|| serialized.strip_prefix("---\r\n"))
        .unwrap_or(serialized);

    let without_end = without_start
        .strip_suffix("\n...")
        .or_else(|| without_start.strip_suffix("\r\n..."))
        .or_else(|| without_start.strip_suffix("...\n"))
        .or_else(|| without_start.strip_suffix("...\r\n"))
        .unwrap_or(without_start);

    without_end.trim_end_matches(['\n', '\r']).to_string()
}

fn strip_opening_delimiter<'a>(content: &'a str, delimiter: &str) -> Option<&'a str> {
    if !content.starts_with(delimiter) {
        return None;
    }

    let mut rest = &content[delimiter.len()..];

    if let Some(stripped) = rest.strip_prefix("\r\n") {
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix('\n') {
        rest = stripped;
    }

    Some(rest)
}

fn extract_frontmatter_block<'a>(content: &'a str, delimiter: &'a str) -> Option<(&'a str, usize)> {
    let mut offset = 0;

    for line in content.split_terminator('\n') {
        let trimmed = line.trim_end_matches('\r');

        if trimmed == delimiter {
            let delimiter_len = line.len();
            let mut body_start = offset + delimiter_len;

            if content.len() > body_start && content.as_bytes()[body_start] == b'\n' {
                body_start += 1;
            }

            let frontmatter_str = &content[..offset];
            return Some((frontmatter_str, body_start));
        }

        offset += line.len() + 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("md-splice")
            .join("tests/fixtures/frontmatter")
            .join(name)
    }

    #[test]
    fn parses_yaml_frontmatter() {
        let content = std::fs::read_to_string(fixture_path("yaml_simple.md")).unwrap();
        let parsed = parse(&content).unwrap();

        assert_eq!(parsed.format, Some(FrontmatterFormat::Yaml));
        let expected: YamlValue = serde_yaml::from_str("title: Sample\nstatus: draft\n").unwrap();
        assert_eq!(parsed.frontmatter, Some(expected));
        assert!(parsed.body.starts_with("# Heading"));
    }

    #[test]
    fn parses_toml_frontmatter() {
        let content = std::fs::read_to_string(fixture_path("toml_simple.md")).unwrap();
        let parsed = parse(&content).unwrap();

        assert_eq!(parsed.format, Some(FrontmatterFormat::Toml));
        let expected: YamlValue = serde_yaml::from_str("title: Sample\nstatus: draft\n").unwrap();
        assert_eq!(parsed.frontmatter, Some(expected));
        assert!(parsed.body.starts_with("# Heading"));
    }

    #[test]
    fn handles_missing_frontmatter() {
        let content = std::fs::read_to_string(fixture_path("no_frontmatter.md")).unwrap();
        let parsed = parse(&content).unwrap();

        assert!(parsed.frontmatter.is_none());
        assert!(parsed.format.is_none());
        assert_eq!(parsed.body, content);
    }

    #[test]
    fn handles_empty_frontmatter() {
        let content = std::fs::read_to_string(fixture_path("empty_frontmatter.md")).unwrap();
        let parsed = parse(&content).unwrap();

        assert_eq!(parsed.format, Some(FrontmatterFormat::Yaml));
        assert_eq!(parsed.frontmatter, Some(YamlValue::Null));
        assert!(parsed.body.starts_with("# Empty"));
    }

    #[test]
    fn errors_on_malformed_frontmatter() {
        let content = std::fs::read_to_string(fixture_path("malformed.md")).unwrap();
        let err = parse(&content).unwrap_err();

        assert!(err
            .to_string()
            .contains("Failed to parse YAML frontmatter at start of document"));
    }
}
