from __future__ import annotations

from textwrap import dedent

import pytest

from md_splice import (
    DeleteOperation,
    InsertOperation,
    InsertPosition,
    ReplaceOperation,
    Selector,
    dumps_operations,
    loads_operations,
)
from md_splice.errors import OperationParseError


OPERATIONS_YAML = dedent(
    """
    - op: insert
      selector:
        select_type: li
        select_contains: Write documentation
      position: before
      content: "- [ ] Ship release"
    - op: delete
      selector:
        select_type: p
        select_contains: TODO
    """
)

OPERATIONS_JSON = dedent(
    """
    [
      {
        "op": "replace",
        "selector": {
          "select_type": "p"
        },
        "content": "Updated paragraph."
      }
    ]
    """
)


def test_loads_operations_from_yaml() -> None:
    operations = loads_operations(OPERATIONS_YAML)

    assert isinstance(operations, list)
    assert isinstance(operations[0], InsertOperation)
    assert operations[0].position is InsertPosition.BEFORE
    assert isinstance(operations[1], DeleteOperation)
    assert operations[1].selector.select_contains == "TODO"


def test_loads_operations_from_json() -> None:
    operations = loads_operations(OPERATIONS_JSON, format="json")

    assert isinstance(operations[0], ReplaceOperation)
    assert operations[0].content == "Updated paragraph."


def test_dumps_operations_round_trip_yaml() -> None:
    ops = [
        InsertOperation(
            selector=Selector(select_type="li", select_contains="Write documentation"),
            content="- [ ] Ship release",
            position=InsertPosition.BEFORE,
        ),
        DeleteOperation(selector=Selector(select_contains="TODO")),
    ]

    rendered = dumps_operations(ops)
    round_tripped = loads_operations(rendered)

    assert round_tripped[0].content == "- [ ] Ship release"
    assert round_tripped[1].selector.select_contains == "TODO"


def test_dumps_operations_json_format() -> None:
    ops = [ReplaceOperation(selector=Selector(select_type="p"), content="Updated.")]

    rendered = dumps_operations(ops, format="json")
    parsed = loads_operations(rendered, format="json")

    assert parsed[0].content == "Updated."


def test_loads_operations_rejects_file_fields() -> None:
    yaml_with_file = dedent(
        """
        - op: insert
          selector:
            select_type: p
          content_file: snippet.md
        """
    )

    with pytest.raises(OperationParseError):
        loads_operations(yaml_with_file)
