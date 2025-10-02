# Changelog

## 0.1.0 (2025-10-02)


### Bug Fixes

* resolve CI failures\n\n- Remove invalid "cli" feature from codex-apply-patch dep\n- Apply rustfmt to tests/integration.rs\n\nNote: Cargo.lock left as-is; CI should resolve with current lock. If lock update is still required, we can run cargo update -p codex-apply-patch in a follow-up. ([84455ac](https://github.com/HautechAI/codex-tools-mcp/commit/84455ace1047b62ef073a3a2cc464db579172492))
