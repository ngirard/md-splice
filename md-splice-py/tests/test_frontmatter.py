from md_splice import FrontmatterFormat, MarkdownDocument


def test_yaml_frontmatter_round_trip():
    source = """---\ntitle: Example\ntags:\n  - alpha\n  - beta\n---\nBody\n"""
    doc = MarkdownDocument.from_string(source)

    frontmatter = doc.frontmatter()
    assert frontmatter == {"title": "Example", "tags": ["alpha", "beta"]}
    assert doc.frontmatter_format() is FrontmatterFormat.YAML


def test_toml_frontmatter_round_trip():
    source = """+++\ntitle = \"Example\"\ncount = 42\n+++\nBody\n"""
    doc = MarkdownDocument.from_string(source)

    frontmatter = doc.frontmatter()
    assert frontmatter == {"title": "Example", "count": 42}
    assert doc.frontmatter_format() is FrontmatterFormat.TOML


def test_empty_frontmatter_maps_to_none():
    source = """---\n---\nBody\n"""
    doc = MarkdownDocument.from_string(source)

    assert doc.frontmatter() is None
    assert doc.frontmatter_format() is FrontmatterFormat.YAML
