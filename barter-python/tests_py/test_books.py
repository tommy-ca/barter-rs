"""Tests for OrderBook bindings."""

import pytest

import barter_python as bp


class TestLevel:
    """Test Level class."""

    def test_new_valid(self):
        """Test creating a valid Level."""
        level = bp.Level(100.5, 2.0)
        assert level.price == "100.5"
        assert level.amount == "2"

    def test_new_invalid_price(self):
        """Test creating Level with invalid price."""
        with pytest.raises(ValueError, match="price must be a positive, finite numeric value"):
            bp.Level(-1.0, 2.0)

        with pytest.raises(ValueError, match="price must be a positive, finite numeric value"):
            bp.Level(0.0, 2.0)

        with pytest.raises(ValueError, match="price must be a positive, finite numeric value"):
            bp.Level(float('nan'), 2.0)

    def test_new_invalid_amount(self):
        """Test creating Level with invalid amount."""
        with pytest.raises(ValueError, match="amount must be a non-negative, finite numeric value"):
            bp.Level(100.0, -1.0)

        with pytest.raises(ValueError, match="amount must be a non-negative, finite numeric value"):
            bp.Level(100.0, float('nan'))

    def test_repr(self):
        """Test string representation."""
        level = bp.Level(100.5, 2.0)
        assert repr(level) == "Level(price=100.5, amount=2)"


class TestOrderBook:
    """Test OrderBook class."""

    def test_new_valid(self):
        """Test creating a valid OrderBook."""
        bids = [(100.0, 1.0), (99.5, 2.0)]
        asks = [(100.5, 1.5), (101.0, 1.0)]
        book = bp.OrderBook(123, bids, asks)

        assert book.sequence == 123
        assert book.time_engine is None

        bids_result = book.bids()
        asks_result = book.asks()

        assert len(bids_result) == 2
        assert len(asks_result) == 2

        # Bids should be sorted descending
        assert bids_result[0][0] == "100"
        assert bids_result[1][0] == "99.5"

        # Asks should be sorted ascending
        assert asks_result[0][0] == "100.5"
        assert asks_result[1][0] == "101"

    def test_new_invalid_bid_price(self):
        """Test creating OrderBook with invalid bid price."""
        with pytest.raises(ValueError, match="bid price must be positive and finite"):
            bp.OrderBook(123, [(-1.0, 1.0)], [(100.5, 1.0)])

    def test_new_invalid_ask_price(self):
        """Test creating OrderBook with invalid ask price."""
        with pytest.raises(ValueError, match="ask price must be positive and finite"):
            bp.OrderBook(123, [(100.0, 1.0)], [(0.0, 1.0)])

    def test_mid_price(self):
        """Test mid-price calculation."""
        bids = [(100.0, 1.0)]
        asks = [(102.0, 1.0)]
        book = bp.OrderBook(123, bids, asks)

        mid = book.mid_price()
        assert mid == "101"

    def test_mid_price_empty(self):
        """Test mid-price when empty."""
        book = bp.OrderBook(123, [], [])

        mid = book.mid_price()
        assert mid is None

    def test_volume_weighted_mid_price(self):
        """Test volume weighted mid-price calculation."""
        bids = [(100.0, 2.0)]
        asks = [(102.0, 1.0)]
        book = bp.OrderBook(123, bids, asks)

        vwm = book.volume_weighted_mid_price()
        # (100 * 1 + 102 * 2) / (1 + 2) = (100 + 204) / 3 = 304 / 3 = 101.333...
        assert vwm == "101.33333333333333333333333333"

    def test_repr(self):
        """Test string representation."""
        bids = [(100.0, 1.0)]
        asks = [(101.0, 1.0)]
        book = bp.OrderBook(123, bids, asks)

        assert repr(book) == "OrderBook(sequence=123, bids=1, asks=1)"


class TestCalculateMidPrice:
    """Test calculate_mid_price function."""

    def test_calculate_mid_price(self):
        """Test mid-price calculation."""
        result = bp.calculate_mid_price(100.0, 102.0)
        assert result == "101"

    def test_calculate_mid_price_invalid(self):
        """Test mid-price with invalid inputs."""
        with pytest.raises(ValueError, match="best_bid_price must be finite"):
            bp.calculate_mid_price(float('nan'), 102.0)

        with pytest.raises(ValueError, match="best_ask_price must be finite"):
            bp.calculate_mid_price(100.0, float('inf'))


class TestCalculateVolumeWeightedMidPrice:
    """Test calculate_volume_weighted_mid_price function."""

    def test_calculate_volume_weighted_mid_price(self):
        """Test volume weighted mid-price calculation."""
        result = bp.calculate_volume_weighted_mid_price(100.0, 2.0, 102.0, 1.0)
        # (100 * 1 + 102 * 2) / (1 + 2) = 304 / 3 = 101.333...
        assert result == "101.33333333333333333333333333"

    def test_calculate_volume_weighted_mid_price_invalid(self):
        """Test volume weighted mid-price with invalid inputs."""
        with pytest.raises(ValueError, match="best_bid_price must be finite"):
            bp.calculate_volume_weighted_mid_price(float('nan'), 2.0, 102.0, 1.0)
