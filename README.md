# Codex Tools MCP Server

A minimal [Model Context Protocol](https://modelcontextprotocol.io/) server implemented in Rust that exposes the `update_plan` and `apply_patch` tools used by the Codex GPT-5 model. It lets developers integrate these tools inside any MCP-aware client without running the Codex CLI.

## Install
Prebuilt binaries are published on the GitHub Releases page for common platforms:

- Linux: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
- macOS: `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

Download `codex-tools-mcp-vX.Y.Z-<triple>.tar.gz` (or `.zip` on Windows) for your platform, extract, and run the binary.

To verify integrity, a combined `codex-tools-mcp-vX.Y.Z-checksums.txt` file is attached to each release.

Tag format to trigger releases: `vX.Y.Z`.

## Build From Source
- Rust toolchain: stable recommended. Tentative MSRV is 1.70+; CI enforces stable and checks 1.70 in allow-failure mode until confirmed.

```bash
cargo build --release
```

The binary communicates over stdio using JSON-RPC 2.0. Launch it from an MCP-compatible host (for example, the MCP Inspector or any tool runner that can spawn stdio-based servers). Run `./target/release/codex-tools-mcp --help` for command-line options (log level, version information).

## Tool Schemas

- `update_plan`: matches the schema defined in `codex-rs/core/src/plan_tool.rs` (required `plan` array with `step` and `status`, optional `explanation`).
- `apply_patch`: matches the JSON variant defined in `codex-rs/core/src/tool_apply_patch.rs` (required `input` string containing the full patch payload).

The `apply_patch` tool reuses the official Codex `codex-apply-patch` crate to parse and apply patches, so file changes are applied exactly as in the CLI. The server streams the CLI-equivalent summary back in the MCP response. `update_plan` returns the acknowledgement "Plan updated".

## Agents SDK Demo

For local testing without exposing an HTTP endpoint, use the OpenAI Agents SDK with the stdio MCP server:

```bash
pip install openai-agents
OPENAI_API_KEY=your_key python3 scripts/agents_demo.py
```

Set `OPENAI_API_KEY` (or export it beforehand) so the Agents SDK can authenticate with OpenAI. This runs the codex MCP binary via stdio and asks it to write `hello world!` into `hello.txt` in the working directory.

## Contributing
See `CONTRIBUTING.md` for Conventional Commits guidance and development notes. PRs should be squash-merged.
