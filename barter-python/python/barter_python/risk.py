"""Pure Python implementation of barter risk management module."""

from __future__ import annotations

from abc import abstractmethod
from typing import Generic, Iterable, Protocol, TypeVar, Union

from .execution import OrderRequestCancel, OrderRequestOpen

# Type variables for generic risk interfaces
ExchangeKey = TypeVar("ExchangeKey")
InstrumentKey = TypeVar("InstrumentKey")
State = TypeVar("State")
OrderRequestType = TypeVar("OrderRequestType", bound=Union[OrderRequestCancel, OrderRequestOpen])


class RiskApproved(Generic[OrderRequestType]):
    """New type that wraps order requests that have passed RiskManager checks."""

    def __init__(self, item: OrderRequestCancel | OrderRequestOpen) -> None:
        self._item = item

    @property
    def item(self) -> OrderRequestCancel | OrderRequestOpen:
        """Get the wrapped item."""
        return self._item

    def into_item(self) -> OrderRequestCancel | OrderRequestOpen:
        """Consume self and return the wrapped item."""
        return self._item

    def __repr__(self) -> str:
        return f"RiskApproved({self._item!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, RiskApproved):
            return NotImplemented
        return self._item == other._item

    def __hash__(self) -> int:
        return hash(self._item)


class RiskRefused(Generic[OrderRequestType]):
    """Type that wraps order requests that have failed RiskManager checks, including the failure reason."""

    def __init__(self, item: OrderRequestCancel | OrderRequestOpen, reason: str) -> None:
        self.item = item
        self.reason = reason

    @classmethod
    def new(cls, item: OrderRequestCancel | OrderRequestOpen, reason: str) -> RiskRefused:
        """Create a new RiskRefused with the given item and reason."""
        return cls(item, reason)

    def into_item(self) -> OrderRequestCancel | OrderRequestOpen:
        """Consume self and return the wrapped item."""
        return self.item

    def __repr__(self) -> str:
        return f"RiskRefused(item={self.item!r}, reason={self.reason!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, RiskRefused):
            return NotImplemented
        return self.item == other.item and self.reason == other.reason

    def __hash__(self) -> int:
        return hash((self.item, self.reason))


class RiskManager(Protocol[ExchangeKey, InstrumentKey]):
    """RiskManager interface that reviews and optionally filters cancel and open order requests.

    For example, a RiskManager implementation may wish to:
    - Filter out orders that would result in too much exposure.
    - Filter out orders that have a too high quantity.
    - Adjust order quantities.
    - Filter out orders that would cross the OrderBook.
    - etc.
    """

    @abstractmethod
    def check(
        self,
        state: State,
        cancels: Iterable[OrderRequestCancel[ExchangeKey, InstrumentKey]],
        opens: Iterable[OrderRequestOpen[ExchangeKey, InstrumentKey]],
    ) -> tuple[
        Iterable[RiskApproved[OrderRequestCancel[ExchangeKey, InstrumentKey]]],
        Iterable[RiskApproved[OrderRequestOpen[ExchangeKey, InstrumentKey]]],
        Iterable[RiskRefused[OrderRequestCancel[ExchangeKey, InstrumentKey]]],
        Iterable[RiskRefused[OrderRequestOpen[ExchangeKey, InstrumentKey]]],
    ]:
        """Check and filter order requests based on risk rules.

        Returns:
            Tuple of (approved_cancels, approved_opens, refused_cancels, refused_opens)
        """
        ...


class DefaultRiskManager(Generic[State]):
    """Naive implementation of the RiskManager interface, approving all orders without any risk checks.

    THIS IS FOR DEMONSTRATION PURPOSES ONLY, NEVER USE FOR REAL TRADING OR IN PRODUCTION.
    """

    def check(
        self,
        state: State,
        cancels: Iterable[OrderRequestCancel[ExchangeKey, InstrumentKey]],
        opens: Iterable[OrderRequestOpen[ExchangeKey, InstrumentKey]],
    ) -> tuple[
        Iterable[RiskApproved[OrderRequestCancel[ExchangeKey, InstrumentKey]]],
        Iterable[RiskApproved[OrderRequestOpen[ExchangeKey, InstrumentKey]]],
        Iterable[RiskRefused[OrderRequestCancel[ExchangeKey, InstrumentKey]]],
        Iterable[RiskRefused[OrderRequestOpen[ExchangeKey, InstrumentKey]]],
    ]:
        """Approve all orders without any checks."""
        approved_cancels = [RiskApproved(cancel) for cancel in cancels]
        approved_opens = [RiskApproved(open_req) for open_req in opens]
        refused_cancels = []
        refused_opens = []

        return approved_cancels, approved_opens, refused_cancels, refused_opens