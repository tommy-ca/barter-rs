"""Unit tests for the risk management module."""

from decimal import Decimal

import pytest

import barter_python as bp
from barter_python import risk


class TestRiskApproved:
    """Test RiskApproved wrapper."""

    def test_creation(self):
        """Test creating a RiskApproved."""
        # Use a mock request object
        request = MockOrderRequest()
        approved = risk.RiskApproved(request)

        assert approved.item == request
        assert approved.into_item() == request

    def test_equality(self):
        """Test equality of RiskApproved."""
        request1 = MockOrderRequest()
        request2 = MockOrderRequest()
        approved1 = risk.RiskApproved(request1)
        approved2 = risk.RiskApproved(request2)

        assert approved1 == approved2


class TestRiskRefused:
    """Test RiskRefused wrapper."""

    def test_creation(self):
        """Test creating a RiskRefused."""
        request = MockOrderRequest()
        refused = risk.RiskRefused.new(request, "Test reason")

        assert refused.item == request
        assert refused.reason == "Test reason"
        assert refused.into_item() == request

    def test_equality(self):
        """Test equality of RiskRefused."""
        request1 = MockOrderRequest()
        request2 = MockOrderRequest()
        refused1 = risk.RiskRefused.new(request1, "reason")
        refused2 = risk.RiskRefused.new(request2, "reason")

        assert refused1 == refused2


class TestDefaultRiskManager:
    """Test DefaultRiskManager implementation."""

    def test_check_approves_all(self):
        """Test that DefaultRiskManager approves all orders."""
        manager = risk.DefaultRiskManager()

        cancels = [MockOrderRequest()]
        opens = [MockOrderRequest()]

        approved_cancels, approved_opens, refused_cancels, refused_opens = (
            manager.check(None, cancels, opens)
        )

        assert len(list(approved_cancels)) == 1
        assert len(list(approved_opens)) == 1
        assert len(list(refused_cancels)) == 0
        assert len(list(refused_opens)) == 0


class TestRiskUtilities:
    """Test bindings for risk utility helpers."""

    def test_calculate_quote_notional(self):
        """Should multiply quantity, price, and contract size."""

        result = risk.calculate_quote_notional(
            Decimal("2.5"), Decimal("102.5"), Decimal("1.5")
        )

        assert result == Decimal("384.375")

    def test_calculate_quote_notional_overflow(self):
        """Should return None when multiplication overflows."""

        huge = Decimal("1e20")

        result = risk.calculate_quote_notional(huge, huge, Decimal(1))

        assert result is None

    @pytest.mark.parametrize(
        ("current", "other", "expected"),
        [
            (Decimal("105"), Decimal("100"), Decimal("0.05")),
            (Decimal("95"), Decimal("100"), Decimal("0.05")),
        ],
    )
    def test_calculate_abs_percent_difference(self, current, other, expected):
        """Should calculate absolute percent difference."""

        result = risk.calculate_abs_percent_difference(current, other)

        assert result == expected

    def test_calculate_abs_percent_difference_zero_other(self):
        """Should return None when dividing by zero."""

        result = risk.calculate_abs_percent_difference(
            Decimal("10"), Decimal("0")
        )

        assert result is None

    @pytest.mark.parametrize(
        ("instrument_delta", "contract_size", "quantity", "side", "expected"),
        [
            (Decimal("1"), Decimal("1"), Decimal("5"), bp.Side.BUY, Decimal("5")),
            (
                Decimal("0.5"),
                Decimal("100"),
                Decimal("2"),
                bp.Side.SELL,
                Decimal("-100"),
            ),
        ],
    )
    def test_calculate_delta(
        self, instrument_delta, contract_size, quantity, side, expected
    ):
        """Should honour directional exposure sign."""

        result = risk.calculate_delta(
            instrument_delta, contract_size, side, quantity
        )

        assert result == expected


class MockOrderRequest:
    """Mock order request for testing."""

    def __init__(self):
        self.key = "mock_key"
        self.state = "mock_state"

    def __eq__(self, other):
        return isinstance(other, MockOrderRequest)

    def __hash__(self):
        return hash(("MockOrderRequest", self.key, self.state))
