"""Unit tests for pure Python integration data structures."""


from barter_python.integration import (
    Field,
    Metric,
    Snapshot,
    SnapUpdates,
    SubscriptionId,
    Tag,
    Value,
)


class TestSubscriptionId:
    def test_creation(self):
        sid = SubscriptionId.new("test-stream")
        assert sid.id == "test-stream"

    def test_equality(self):
        sid1 = SubscriptionId.new("test-stream")
        sid2 = SubscriptionId.new("test-stream")
        sid3 = SubscriptionId.new("other-stream")
        assert sid1 == sid2
        assert sid1 != sid3

    def test_str_repr(self):
        sid = SubscriptionId.new("test-stream")
        assert str(sid) == "test-stream"
        assert repr(sid) == "SubscriptionId('test-stream')"


class TestMetric:
    def test_creation(self):
        tags = [Tag.new("key1", "value1")]
        fields = [Field.new("field1", Value.float(1.0))]
        metric = Metric.new("test_metric", 1234567890, tags, fields)
        assert metric.name == "test_metric"
        assert metric.time == 1234567890
        assert metric.tags == tags
        assert metric.fields == fields

    def test_creation_defaults(self):
        metric = Metric.new("test_metric", 1234567890)
        assert metric.name == "test_metric"
        assert metric.time == 1234567890
        assert metric.tags == []
        assert metric.fields == []

    def test_equality(self):
        tags = [Tag.new("key1", "value1")]
        fields = [Field.new("field1", Value.float(1.0))]
        metric1 = Metric.new("test_metric", 1234567890, tags, fields)
        metric2 = Metric.new("test_metric", 1234567890, tags, fields)
        metric3 = Metric.new("other_metric", 1234567890, tags, fields)
        assert metric1 == metric2
        assert metric1 != metric3

    def test_str_repr(self):
        metric = Metric.new("test_metric", 1234567890)
        assert "Metric(" in repr(metric)


class TestTag:
    def test_creation(self):
        tag = Tag.new("key1", "value1")
        assert tag.key == "key1"
        assert tag.value == "value1"

    def test_equality(self):
        tag1 = Tag.new("key1", "value1")
        tag2 = Tag.new("key1", "value1")
        tag3 = Tag.new("key2", "value1")
        assert tag1 == tag2
        assert tag1 != tag3

    def test_str_repr(self):
        tag = Tag.new("key1", "value1")
        assert str(tag) == "key1=value1"
        assert "Tag(" in repr(tag)

    def test_ordering(self):
        tag1 = Tag.new("a", "1")
        tag2 = Tag.new("b", "1")
        assert tag1 < tag2


class TestField:
    def test_creation(self):
        value = Value.float(1.0)
        field = Field.new("field1", value)
        assert field.key == "field1"
        assert field.value == value

    def test_equality(self):
        value = Value.float(1.0)
        field1 = Field.new("field1", value)
        field2 = Field.new("field1", value)
        field3 = Field.new("field2", value)
        assert field1 == field2
        assert field1 != field3

    def test_str_repr(self):
        field = Field.new("field1", Value.float(1.0))
        assert "Field(" in repr(field)


class TestValue:
    def test_float(self):
        value = Value.float(1.5)
        assert value.kind == "float"
        assert value.value == 1.5

    def test_int(self):
        value = Value.int_value(-42)
        assert value.kind == "int"
        assert value.value == -42

    def test_uint(self):
        value = Value.uint_value(42)
        assert value.kind == "uint"
        assert value.value == 42

    def test_bool(self):
        value = Value.bool_value(True)
        assert value.kind == "bool"
        assert value.value is True

    def test_string(self):
        value = Value.string("hello")
        assert value.kind == "string"
        assert value.value == "hello"

    def test_equality(self):
        v1 = Value.float(1.5)
        v2 = Value.float(1.5)
        v3 = Value.float(2.0)
        assert v1 == v2
        assert v1 != v3

    def test_str_repr(self):
        value = Value.float(1.5)
        assert str(value) == "1.5"
        assert repr(value) == "Value.float(1.5)"


class TestSnapshot:
    def test_creation(self):
        snapshot = Snapshot.new("test_value")
        assert snapshot.value == "test_value"

    def test_as_ref(self):
        snapshot = Snapshot.new("test_value")
        ref = snapshot.as_ref()
        assert ref == snapshot

    def test_map(self):
        snapshot = Snapshot.new(5)
        mapped = snapshot.map(lambda x: x * 2)
        assert mapped.value == 10

    def test_equality(self):
        s1 = Snapshot.new("test")
        s2 = Snapshot.new("test")
        s3 = Snapshot.new("other")
        assert s1 == s2
        assert s1 != s3

    def test_str_repr(self):
        snapshot = Snapshot.new("test")
        assert "Snapshot(" in repr(snapshot)


class TestSnapUpdates:
    def test_creation(self):
        snapshot = "snapshot_data"
        updates = ["update1", "update2"]
        snap_updates = SnapUpdates.new(snapshot, updates)
        assert snap_updates.snapshot == snapshot
        assert snap_updates.updates == updates

    def test_equality(self):
        su1 = SnapUpdates.new("snap", ["u1"])
        su2 = SnapUpdates.new("snap", ["u1"])
        su3 = SnapUpdates.new("other", ["u1"])
        assert su1 == su2
        assert su1 != su3

    def test_str_repr(self):
        su = SnapUpdates.new("snap", ["u1"])
        assert "SnapUpdates(" in repr(su)
