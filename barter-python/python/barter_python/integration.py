"""Pure Python implementation of barter-integration data structures."""

from __future__ import annotations

from typing import Generic, Optional, TypeVar

T = TypeVar("T")
SnapshotType = TypeVar("SnapshotType")
UpdatesType = TypeVar("UpdatesType")


class SubscriptionId:
    """Unique identifier for a stream subscription."""

    def __init__(self, id: str) -> None:
        self._id = id

    @property
    def id(self) -> str:
        return self._id

    @classmethod
    def new(cls, id: str) -> SubscriptionId:
        return cls(id)

    def __str__(self) -> str:
        return self._id

    def __repr__(self) -> str:
        return f"SubscriptionId({self._id!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, SubscriptionId):
            return NotImplemented
        return self._id == other._id

    def __hash__(self) -> int:
        return hash(self._id)


class Metric:
    """Metric data structure."""

    def __init__(self, name: str, time: int, tags: list[Tag], fields: list[Field]) -> None:
        self.name = name
        self.time = time
        self.tags = tags
        self.fields = fields

    @classmethod
    def new(cls, name: str, time: int, tags: Optional[list[Tag]] = None, fields: Optional[list[Field]] = None) -> Metric:
        return cls(name, time, tags or [], fields or [])

    def __str__(self) -> str:
        return f"Metric(name={self.name}, time={self.time}, tags={len(self.tags)}, fields={len(self.fields)})"

    def __repr__(self) -> str:
        return (
            f"Metric("
            f"name={self.name!r}, "
            f"time={self.time!r}, "
            f"tags={self.tags!r}, "
            f"fields={self.fields!r}"
            f")"
        )

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Metric):
            return NotImplemented
        return (
            self.name == other.name
            and self.time == other.time
            and self.tags == other.tags
            and self.fields == other.fields
        )

    def __hash__(self) -> int:
        return hash((self.name, self.time, tuple(self.tags), tuple(self.fields)))


class Tag:
    """Metric tag."""

    def __init__(self, key: str, value: str) -> None:
        self.key = key
        self.value = value

    @classmethod
    def new(cls, key: str, value: str) -> Tag:
        return cls(key, value)

    def __str__(self) -> str:
        return f"{self.key}={self.value}"

    def __repr__(self) -> str:
        return f"Tag(key={self.key!r}, value={self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Tag):
            return NotImplemented
        return self.key == other.key and self.value == other.value

    def __hash__(self) -> int:
        return hash((self.key, self.value))

    def __lt__(self, other: Tag) -> bool:
        if not isinstance(other, Tag):
            return NotImplemented
        return (self.key, self.value) < (other.key, other.value)


class Field:
    """Metric field."""

    def __init__(self, key: str, value: Value) -> None:
        self.key = key
        self.value = value

    @classmethod
    def new(cls, key: str, value: Value) -> Field:
        return cls(key, value)

    def __str__(self) -> str:
        return f"{self.key}={self.value}"

    def __repr__(self) -> str:
        return f"Field(key={self.key!r}, value={self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Field):
            return NotImplemented
        return self.key == other.key and self.value == other.value

    def __hash__(self) -> int:
        return hash((self.key, self.value))


class Value:
    """Metric value enum."""

    def __init__(self, kind: str, value):
        self.kind = kind
        self.value = value

    @classmethod
    def float(cls, value: float) -> Value:
        return cls("float", value)

    @classmethod
    def int_value(cls, value: int) -> Value:
        return cls("int", value)

    @classmethod
    def uint_value(cls, value: int) -> Value:
        return cls("uint", value)

    @classmethod
    def bool_value(cls, value: bool) -> Value:
        return cls("bool", value)

    @classmethod
    def string(cls, value: str) -> Value:
        return cls("string", value)

    def __str__(self) -> str:
        return str(self.value)

    def __repr__(self) -> str:
        return f"Value.{self.kind}({self.value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Value):
            return NotImplemented
        return self.kind == other.kind and self.value == other.value

    def __hash__(self) -> int:
        return hash((self.kind, self.value))


class Snapshot(Generic[T]):
    """Snapshot wrapper."""

    def __init__(self, value: T) -> None:
        self._value = value

    @property
    def value(self) -> T:
        return self._value

    @classmethod
    def new(cls, value: T) -> Snapshot[T]:
        return cls(value)

    def as_ref(self) -> Snapshot[T]:
        return self

    def map(self, op):
        return Snapshot(op(self._value))

    def __str__(self) -> str:
        return f"Snapshot({self._value})"

    def __repr__(self) -> str:
        return f"Snapshot({self._value!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Snapshot):
            return NotImplemented
        return self._value == other._value

    def __hash__(self) -> int:
        return hash(self._value)


class SnapUpdates(Generic[SnapshotType, UpdatesType]):
    """Snapshot with updates."""

    def __init__(self, snapshot: SnapshotType, updates: UpdatesType) -> None:
        self.snapshot = snapshot
        self.updates = updates

    @classmethod
    def new(cls, snapshot: SnapshotType, updates: UpdatesType) -> SnapUpdates[SnapshotType, UpdatesType]:
        return cls(snapshot, updates)

    def __str__(self) -> str:
        return f"SnapUpdates(snapshot={self.snapshot}, updates={self.updates})"

    def __repr__(self) -> str:
        return f"SnapUpdates(snapshot={self.snapshot!r}, updates={self.updates!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, SnapUpdates):
            return NotImplemented
        return self.snapshot == other.snapshot and self.updates == other.updates

    def __hash__(self) -> int:
        return hash((self.snapshot, self.updates))
