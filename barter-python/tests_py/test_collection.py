from __future__ import annotations

import pytest

import barter_python as bp


def test_none_one_or_many_construction() -> None:
    empty = bp.NoneOneOrMany()
    assert empty.is_none
    assert len(empty) == 0
    assert empty.to_list() == []

    single = bp.NoneOneOrMany("alpha")
    assert single.is_one
    assert list(single) == ["alpha"]

    many = bp.NoneOneOrMany([1, 2, 3])
    assert many.is_many
    assert many.to_list() == [1, 2, 3]
    assert "Many" in repr(many)


def test_one_or_many_construction() -> None:
    one = bp.OneOrMany("beta")
    assert one.is_one
    assert one.to_list() == ["beta"]

    many = bp.OneOrMany(("gamma", "delta"))
    assert many.is_many
    assert list(many) == ["gamma", "delta"]


def test_one_or_many_empty_iterable_raises() -> None:
    with pytest.raises(ValueError):
        bp.OneOrMany([])
