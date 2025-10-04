# Pure Python Port of Barter Engine Module (2025-10-04)

## Objective
- Port the Rust `barter::engine` module to pure Python, maintaining feature parity and API compatibility.
- Provide core trading engine logic for processing market events, managing orders, positions, and executing strategies.
- Follow TDD principles with 80% focus on implementation, 20% on testing.

## Requirements
- Implement EngineState structures for tracking global, instrument, order, position, and trading state.
- Port engine actions for generating orders, closing positions, sending requests, and canceling orders.
- Integrate with existing pure Python components (strategy, risk, execution, data).
- Support both live and backtest modes with appropriate abstractions.
- Maintain SOLID/KISS/DRY principles and modern Python practices.

## Components to Port

### Engine State Structures
- `GlobalData`: Global engine state (default implementation for basic data).
- `InstrumentMarketData`: Market data state per instrument (prices, order books, etc.).
- `InstrumentState`: Complete state for a single instrument (position, orders, market data).
- `OrderState`: State management for individual orders (open, canceled, filled, etc.).
- `Position`: Position tracking with quantity, entry price, and PnL.
- `TradingState`: Overall trading state (enabled/disabled).
- `EngineState`: Top-level state container combining all state types.

### Engine Actions
- `generate_algo_orders`: Generate algorithmic orders based on strategy and current state.
- `close_positions`: Generate orders to close open positions.
- `send_requests`: Send order requests to execution layer.
- `cancel_orders`: Cancel existing orders based on filters.

### Engine Core
- `Engine`: Main engine struct coordinating state and actions.
- Support for different execution modes (live vs backtest).
- Integration with risk managers for order validation.
- Event-driven processing of market data and account events.

## Implementation Notes
- Use dataclasses and protocols for clean, type-safe structures.
- Implement proper state transitions and validation.
- Integrate with existing pure Python strategy and risk interfaces.
- Maintain compatibility with existing backtest and live system workflows.
- Use async/await for event processing where appropriate.
- Ensure account events (snapshots, balance updates, order state changes, trades) mutate EngineState balances, positions, and orders consistently with Rust implementation.

## Testing
- Port all Rust unit tests to Python equivalents.
- Add integration tests for full engine workflows.
- Test state transitions and action generation.
- Ensure compatibility with existing pure Python components.

## Dependencies
- Existing barter_python modules (strategy, risk, execution, data, instrument).
- `asyncio` for async event processing.
- `decimal` for financial calculations.
- `datetime` for time handling.</content>
</xai:function_call: <xai:function_call name="todowrite">
<parameter name="todos">[{"status":"completed","id":"create_engine_spec"}]
