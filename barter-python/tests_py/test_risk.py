"""Unit tests for the risk management module."""

from decimal import Decimal

import pytest

import barter_python as bp
from barter_python import risk
import barter_python.barter_python as core


class TestRiskApproved:
    """Test RiskApproved wrapper."""

    def test_creation(self):
        """Test creating a RiskApproved."""
        request = make_open_request()
        approved = risk.RiskApproved(request)

        assert isinstance(approved, core.RiskApproved)
        item = approved.item
        assert repr(item) == repr(request)
        assert repr(approved.into_item()) == repr(request)
        assert type(approved).__module__ == "barter_python"

    def test_equality(self):
        """Test equality of RiskApproved."""
        request1 = make_open_request()
        request2 = make_open_request()
        approved1 = risk.RiskApproved(request1)
        approved2 = risk.RiskApproved(request2)

        assert approved1 == approved2


class TestRiskRefused:
    """Test RiskRefused wrapper."""

    def test_creation(self):
        """Test creating a RiskRefused."""
        request = make_cancel_request()
        refused = risk.RiskRefused.new(request, "Test reason")

        assert isinstance(refused, core.RiskRefused)
        item = refused.item
        assert repr(item) == repr(request)
        assert refused.reason == "Test reason"
        assert repr(refused.into_item()) == repr(request)
        assert type(refused).__module__ == "barter_python"

    def test_equality(self):
        """Test equality of RiskRefused."""
        request1 = make_cancel_request()
        request2 = make_cancel_request()
        refused1 = risk.RiskRefused.new(request1, "reason")
        refused2 = risk.RiskRefused.new(request2, "reason")

        assert refused1 == refused2


class TestDefaultRiskManager:
    """Test DefaultRiskManager implementation."""

    def test_check_approves_all(self):
        """Test that DefaultRiskManager approves all orders."""
        manager = risk.DefaultRiskManager()

        cancels = [make_cancel_request()]
        opens = [make_open_request()]

        approved_cancels, approved_opens, refused_cancels, refused_opens = manager.check(
            None, cancels, opens
        )

        approved_cancels = list(approved_cancels)
        approved_opens = list(approved_opens)
        refused_cancels = list(refused_cancels)
        refused_opens = list(refused_opens)

        assert len(approved_cancels) == 1
        assert len(approved_opens) == 1
        assert len(refused_cancels) == 0
        assert len(refused_opens) == 0
        assert isinstance(approved_cancels[0], core.RiskApproved)
        assert isinstance(approved_opens[0], core.RiskApproved)


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


def make_order_key() -> bp.OrderKey:
    """Helper to build a deterministic order key."""

    exchange = 1
    instrument = 99
    strategy = bp.StrategyId.new("strategy-alpha")
    cid = bp.ClientOrderId.new("cid-100")
    return bp.OrderKey(exchange, instrument, strategy, cid)


def make_open_request() -> bp.OrderRequestOpen:
    """Create a simple open request for testing."""

    key = make_order_key()
    return bp.OrderRequestOpen(key, "buy", 100.0, 1.5)


def make_cancel_request() -> bp.OrderRequestCancel:
    """Create a simple cancel request for testing."""

    key = make_order_key()
    return bp.OrderRequestCancel(key)
