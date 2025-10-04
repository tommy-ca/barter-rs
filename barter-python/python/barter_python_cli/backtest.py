"""Command-line interface for running Barter historic backtests via Python bindings."""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Iterable, Sequence
from pathlib import Path

import barter_python as bp

DEFAULT_RISK_FREE_RETURN = 0.05
DEFAULT_INTERVAL = "daily"


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Run a historic backtest using the Barter trading engine bindings "
            "and emit the resulting trading summary in JSON form."
        )
    )

    parser.add_argument(
        "--config",
        type=Path,
        required=True,
        help="Path to a system configuration JSON file.",
    )
    parser.add_argument(
        "--market-data",
        type=Path,
        required=True,
        help="Path to a market data JSON file containing stream events.",
    )
    parser.add_argument(
        "--risk-free-return",
        type=float,
        default=DEFAULT_RISK_FREE_RETURN,
        help=(
            "Annualised risk-free return used for ratio calculations "
            f"(default: {DEFAULT_RISK_FREE_RETURN})."
        ),
    )
    parser.add_argument(
        "--interval",
        type=str,
        default=DEFAULT_INTERVAL,
        choices=("daily", "annual-252", "annual-365"),
        help=(
            "Interval used to annualise summary metrics. Allowed values: "
            "'daily', 'annual-252', 'annual-365' (default: daily)."
        ),
    )
    parser.add_argument(
        "--format",
        choices=("json",),
        default="json",
        help="Output format for the resulting summary (currently only JSON).",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Pretty-print the JSON output with indentation.",
    )
    parser.add_argument(
        "--log-filter",
        help=(
            "Optional tracing filter specification passed to the underlying Rust "
            "tracing subscriber (for example: 'barter_python=debug')."
        ),
    )

    return parser


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    """Parse command-line arguments for the backtest CLI."""

    parser = _build_parser()
    return parser.parse_args(argv)


def run_backtest(
    *,
    config_path: Path,
    market_data_path: Path,
    risk_free_return: float,
    interval: str,
) -> bp.TradingSummary:
    """Execute a historic backtest and return the resulting trading summary."""

    config = bp.SystemConfig.from_json(str(config_path))
    summary = bp.run_historic_backtest(
        config,
        str(market_data_path),
        risk_free_return=risk_free_return,
        interval=interval,
    )
    return summary


def format_summary(summary: bp.TradingSummary, *, pretty: bool) -> str:
    """Render a trading summary to JSON."""

    indent = 2 if pretty else None
    summary_dict = summary.to_dict()
    return json.dumps(summary_dict, indent=indent, default=str)


def main(argv: Iterable[str] | None = None) -> int:
    """CLI entry point returning a process-like exit code."""

    args = parse_args(list(argv) if argv is not None else None)

    if args.log_filter:
        bp.init_tracing(filter=args.log_filter, ansi=sys.stdout.isatty())

    summary = run_backtest(
        config_path=args.config,
        market_data_path=args.market_data,
        risk_free_return=args.risk_free_return,
        interval=args.interval,
    )

    output = format_summary(summary, pretty=args.pretty)
    sys.stdout.write(output)
    if args.pretty:
        sys.stdout.write("\n")

    return 0


__all__ = ["main", "parse_args", "run_backtest", "format_summary"]
