# Contributing

Thanks for your interest in contributing to codex-tools-mcp!

## Development
- Install the Rust toolchain (stable recommended). We aim for MSRV around 1.70+ but CI enforces stable only for now; an additional matrix entry checks 1.70 in allow-failure mode until confirmed.
- Run `cargo fmt` and `cargo clippy -D warnings` before sending a PR.
- Run tests with `cargo test`.

## Conventional Commits
We use Conventional Commits for PR titles. Common types:
- feat, fix, perf, refactor, chore, docs, test, ci, build, revert

Optional scopes to help clarity: cli, server, tools, ci, docs, deps.

Examples:
- feat(server): add config flag to enable verbose logs
- fix(cli)!: change default output format to json (BREAKING)
- chore(ci): enable multi-arch release assets

A breaking change can be indicated with a `!` after the type/scope or by adding a `BREAKING CHANGE:` footer.

PRs should be squash-merged. Please keep the PR title in Conventional Commits format, as it will become the squash commit message.
