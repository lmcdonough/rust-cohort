"""Integration tests for the rust_json_parser Python bindings.

Run with: pytest tests/test_python_integration.py

Requires the extension module to be built and installed first, e.g.:
    python -m venv .venv && source .venv/bin/activate
    pip install maturin pytest
    maturin develop
"""

from pathlib import Path

import pytest

from rust_json_parser import dumps, parse_json, parse_json_file


# ---------------------------------------------------------------------------
# parse_json: primitives
# ---------------------------------------------------------------------------


def test_parse_null():
    assert parse_json("null") is None


@pytest.mark.parametrize("literal,expected", [("true", True), ("false", False)])
def test_parse_bool(literal, expected):
    assert parse_json(literal) is expected


@pytest.mark.parametrize(
    "literal,expected",
    [
        ("0", 0.0),
        ("42", 42.0),
        ("-7", -7.0),
        ("3.14", 3.14),
        ("-0.5", -0.5),
    ],
)
def test_parse_number(literal, expected):
    result = parse_json(literal)
    assert isinstance(result, float)
    assert result == expected


def test_parse_string():
    assert parse_json('"hello"') == "hello"


def test_parse_string_with_escapes():
    assert parse_json(r'"line1\nline2"') == "line1\nline2"
    assert parse_json(r'"quote: \""') == 'quote: "'
    assert parse_json(r'"back\\slash"') == "back\\slash"


# ---------------------------------------------------------------------------
# parse_json: containers
# ---------------------------------------------------------------------------


def test_parse_empty_array():
    assert parse_json("[]") == []


def test_parse_array_of_primitives():
    assert parse_json("[1, 2, 3]") == [1.0, 2.0, 3.0]


def test_parse_mixed_array():
    assert parse_json('[1, "two", true, null]') == [1.0, "two", True, None]


def test_parse_empty_object():
    assert parse_json("{}") == {}


def test_parse_flat_object():
    assert parse_json('{"name": "Alice", "age": 30}') == {"name": "Alice", "age": 30.0}


def test_parse_nested():
    data = parse_json('{"users": [{"id": 1}, {"id": 2}]}')
    assert data == {"users": [{"id": 1.0}, {"id": 2.0}]}


# ---------------------------------------------------------------------------
# parse_json: errors (ValueError, matching stdlib json)
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "bad_input",
    [
        "",
        "{",
        "[1,",
        "{'single': 'quotes'}",
        '{"missing": }',
        "nope",
    ],
)
def test_parse_invalid_raises_value_error(bad_input):
    with pytest.raises(ValueError):
        parse_json(bad_input)


# ---------------------------------------------------------------------------
# parse_json_file
# ---------------------------------------------------------------------------


def test_parse_file_round_trip(tmp_path: Path):
    path = tmp_path / "sample.json"
    path.write_text('{"ok": true, "n": 1}')
    assert parse_json_file(str(path)) == {"ok": True, "n": 1.0}


def test_parse_file_missing_raises_io_error(tmp_path: Path):
    missing = tmp_path / "does-not-exist.json"
    with pytest.raises((IOError, OSError)):
        parse_json_file(str(missing))


def test_parse_file_invalid_json_raises_value_error(tmp_path: Path):
    path = tmp_path / "bad.json"
    path.write_text("{not json")
    with pytest.raises(ValueError):
        parse_json_file(str(path))


# ---------------------------------------------------------------------------
# dumps: compact output
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "obj,expected",
    [
        (None, "null"),
        (True, "true"),
        (False, "false"),
        (42, "42"),
        (3.5, "3.5"),
        ("hi", '"hi"'),
        ([], "[]"),
        ({}, "{}"),
        ([1, 2, 3], "[1,2,3]"),
    ],
)
def test_dumps_compact(obj, expected):
    assert dumps(obj) == expected


def test_dumps_escapes_strings():
    assert dumps('he said "hi"') == r'"he said \"hi\""'
    assert dumps("line1\nline2") == r'"line1\nline2"'
    assert dumps("tab\there") == r'"tab\there"'


def test_dumps_escapes_object_keys():
    assert dumps({'quote"key': 1}) == r'{"quote\"key":1}'


# ---------------------------------------------------------------------------
# dumps: pretty output
# ---------------------------------------------------------------------------


def test_dumps_indent_array():
    assert dumps([1, 2], indent=2) == "[\n  1,\n  2\n]"


def test_dumps_indent_object_single_key():
    assert dumps({"a": 1}, indent=2) == '{\n  "a": 1\n}'


def test_dumps_indent_empty_containers():
    assert dumps([], indent=2) == "[]"
    assert dumps({}, indent=2) == "{}"


def test_dumps_indent_nested():
    out = dumps({"nums": [1, 2]}, indent=2)
    assert out == '{\n  "nums": [\n    1,\n    2\n  ]\n}'


# ---------------------------------------------------------------------------
# dumps: unsupported types
# ---------------------------------------------------------------------------


def test_dumps_unsupported_type_raises_value_error():
    import datetime

    with pytest.raises(ValueError):
        dumps(datetime.datetime(2026, 1, 1))


# ---------------------------------------------------------------------------
# Round-trip: parse -> dumps -> parse
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "source",
    [
        '{"name": "Alice", "age": 30}',
        "[1, 2, 3]",
        '{"nested": {"deep": [true, false, null]}}',
        '{"empty_obj": {}, "empty_arr": []}',
    ],
)
def test_round_trip_preserves_structure(source):
    parsed = parse_json(source)
    reserialized = dumps(parsed)
    # HashMap key order is not stable, so compare by re-parsing
    assert parse_json(reserialized) == parsed
