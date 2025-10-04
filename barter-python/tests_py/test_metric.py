"""Tests for metric bindings."""

import pytest

from barter_python import Field, Metric, Tag, Value


class TestTag:
    """Test Tag binding."""

    def test_construction(self):
        """Test Tag construction."""
        tag = Tag("key", "value")
        assert tag.key == "key"
        assert tag.value == "value"

    def test_equality(self):
        """Test Tag equality."""
        tag1 = Tag("key", "value")
        tag2 = Tag("key", "value")
        tag3 = Tag("key", "different")

        assert tag1 == tag2
        assert tag1 != tag3


class TestValue:
    """Test Value binding."""

    def test_float(self):
        """Test float Value."""
        val = Value.float(3.14)
        assert val.is_float()
        assert val.as_float() == 3.14
        assert not val.is_int()
        assert not val.is_uint()
        assert not val.is_bool()
        assert not val.is_string()

    def test_int(self):
        """Test int Value."""
        val = Value.int(-42)
        assert val.is_int()
        assert val.as_int() == -42
        assert not val.is_float()

    def test_uint(self):
        """Test uint Value."""
        val = Value.uint(42)
        assert val.is_uint()
        assert val.as_uint() == 42
        assert not val.is_float()

    def test_bool(self):
        """Test bool Value."""
        val = Value.bool(True)
        assert val.is_bool()
        assert val.as_bool() is True
        assert not val.is_float()

    def test_string(self):
        """Test string Value."""
        val = Value.string("hello")
        assert val.is_string()
        assert val.as_string() == "hello"
        assert not val.is_float()

    def test_equality(self):
        """Test Value equality."""
        val1 = Value.float(3.14)
        val2 = Value.float(3.14)
        val3 = Value.float(2.71)

        assert val1 == val2
        assert val1 != val3

    def test_type_errors(self):
        """Test type errors when accessing wrong variant."""
        val = Value.float(3.14)

        with pytest.raises(TypeError):
            val.as_int()

        with pytest.raises(TypeError):
            val.as_string()


class TestField:
    """Test Field binding."""

    def test_construction(self):
        """Test Field construction."""
        val = Value.int(42)
        field = Field("test_field", val)

        assert field.key == "test_field"
        assert field.value == val

    def test_equality(self):
        """Test Field equality."""
        val1 = Value.float(1.0)
        val2 = Value.float(1.0)
        val3 = Value.float(2.0)

        field1 = Field("key", val1)
        field2 = Field("key", val2)
        field3 = Field("key", val3)

        assert field1 == field2
        assert field1 != field3


class TestMetric:
    """Test Metric binding."""

    def test_construction(self):
        """Test Metric construction."""
        tags = [Tag("env", "test"), Tag("version", "1.0")]
        fields = [Field("cpu", Value.float(85.5)), Field("memory", Value.uint(1024))]

        metric = Metric("system_metrics", 1234567890, tags, fields)

        assert metric.name == "system_metrics"
        assert metric.time == 1234567890
        assert len(metric.tags) == 2
        assert len(metric.fields) == 2

        # Check tags
        assert metric.tags[0].key == "env"
        assert metric.tags[0].value == "test"
        assert metric.tags[1].key == "version"
        assert metric.tags[1].value == "1.0"

        # Check fields
        assert metric.fields[0].key == "cpu"
        assert metric.fields[0].value.as_float() == 85.5
        assert metric.fields[1].key == "memory"
        assert metric.fields[1].value.as_uint() == 1024

    def test_empty_tags_fields(self):
        """Test Metric with empty tags and fields."""
        metric = Metric("empty_metric", 0, [], [])

        assert metric.name == "empty_metric"
        assert metric.time == 0
        assert len(metric.tags) == 0
        assert len(metric.fields) == 0
