"""Selector data structure tests."""

import re

import pytest

from md_splice.errors import ConflictingScopeError, InvalidRegexError
from md_splice.types import Selector


def test_selector_accepts_compiled_regex():
    pattern = re.compile(r"hello", re.IGNORECASE)
    selector = Selector(
        select_type="heading",
        select_contains="Hello",
        select_regex=pattern,
        select_ordinal=2,
    )
    assert selector.select_regex is pattern


def test_selector_compiles_regex_string():
    selector = Selector(select_regex=r"foo?bar")
    assert selector.select_regex.pattern == r"foo?bar"


def test_selector_invalid_regex_raises():
    with pytest.raises(InvalidRegexError):
        Selector(select_regex="[")


def test_selector_conflicting_scopes_raise():
    base = Selector(select_type="heading")
    with pytest.raises(ConflictingScopeError):
        Selector(after=base, within=base)
