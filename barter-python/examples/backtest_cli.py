#!/usr/bin/env python3

"""Command-line entry point demonstrating the Barter Python bindings."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import barter_python as bp


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run a historic backtest using the Barter trading engine bindings and "
            "emit the resulting trading summary in JSON form."
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
        default=0.05,
        help="Annualised risk-free return used for ratio calculations (default: 0.05).",
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
            "Optional tracing filter specification passed to the underlying "
            "Rust tracing subscriber (for example: 'barter_python=debug')."
        ),
    )

    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)

    if args.log_filter:
        bp.init_tracing(filter=args.log_filter, ansi=sys.stdout.isatty())

    config = bp.SystemConfig.from_json(str(args.config))
    summary = bp.run_historic_backtest(
        config,
        str(args.market_data),
        risk_free_return=args.risk_free_return,
    )

    summary_dict = summary.to_dict()

    indent = 2 if args.pretty else None
    json.dump(summary_dict, sys.stdout, indent=indent, default=str)

    if indent is not None:
        sys.stdout.write("\n")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
