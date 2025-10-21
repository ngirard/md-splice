from md_splice import (
    MarkdownDocument,
    MdSpliceError,
    NodeNotFoundError,
    __version__,
    dumps_operations,
    loads_operations,
)


def test_version_exposed():
    assert __version__ == "0.5.0"


def test_from_string_and_render_round_trip():
    source = "# Title\n\nHello world.\n"
    doc = MarkdownDocument.from_string(source)
    assert isinstance(doc, MarkdownDocument)
    assert doc.render().rstrip("\n") == source.rstrip("\n")


def test_frontmatter_without_block_returns_none():
    doc = MarkdownDocument.from_string("No frontmatter here\n")
    assert doc.frontmatter() is None


def test_error_type_exposed():
    assert issubclass(MdSpliceError, Exception)


def test_operation_serializers_available() -> None:
    assert callable(loads_operations)
    assert callable(dumps_operations)


def test_error_docstrings_present() -> None:
    assert "md-splice Python bindings" in MdSpliceError.__doc__
    assert "selector fails" in NodeNotFoundError.__doc__
