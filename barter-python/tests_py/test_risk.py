"""Unit tests for the risk management module."""

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

        approved_cancels, approved_opens, refused_cancels, refused_opens = manager.check(
            None, cancels, opens
        )

        assert len(list(approved_cancels)) == 1
        assert len(list(approved_opens)) == 1
        assert len(list(refused_cancels)) == 0
        assert len(list(refused_opens)) == 0


class MockOrderRequest:
    """Mock order request for testing."""

    def __init__(self):
        self.key = "mock_key"
        self.state = "mock_state"

    def __eq__(self, other):
        return isinstance(other, MockOrderRequest)

    def __hash__(self):
        return hash(("MockOrderRequest", self.key, self.state))
