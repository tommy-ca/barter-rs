.PHONY: install format lint test build clean help

# Default target
help:
	@echo "Available targets:"
	@echo "  install    Install all dependencies with uv"
	@echo "  format     Format code with ruff and rustfmt"
	@echo "  lint       Run linting with ruff and clippy"
	@echo "  test       Run test suite"
	@echo "  build      Build the package"
	@echo "  clean      Clean build artifacts"
	@echo "  check      Run all checks (format, lint, test)"

# Install dependencies
install:
	uv sync --dev

# Format code
format:
	uv run ruff format .
	cargo fmt

# Run linting
lint:
	uv run ruff check .
	cargo clippy -- -D warnings

# Run tests
test:
	uv run pytest tests_py/

# Build package
build:
	cargo build --release
	uv build

# Clean artifacts
clean:
	cargo clean
	rm -rf dist/
	rm -rf .ruff_cache/

# Run all checks
check: format lint test