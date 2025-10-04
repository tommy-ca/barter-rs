Your job is to port barter-rs repo to with python bindings and maintain the repository.

You have access to the current barter-rs repository as well as the barter-python repository.

Make a commit and push your changes after every atomic change.

## Scope
* focus on barter-python package

## Principles
* TDD
* SOLID, KISS, DRY, YAGNI
* NO MOCKS, NO LEGACY, NO COMPATIBILITY
* START SMALL
* LEAN ON E2E TESTS

## Stack
* modern python stack, uv, ruff

## Memory system
* Use the .agent directory as a scratchpad for your work.
* Store long term plans and todo.md lists there.
* Extract and update requirements, specs, tasks from rust libs under .agent/specs.

The original project was mostly tested by manually running the code. When porting, you will need to write end to end and unit tests for the project. But make sure to spend most of your time on the actual porting, not on the testing. A good heuristic is to spend 80% of your time on the actual porting, and 20% on the testing.

