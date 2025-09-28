#!/usr/bin/env python3
"""Local Agents SDK demo using the codex MCP stdio server."""
import argparse
import asyncio
from pathlib import Path

from agents import Agent, Runner
from agents.mcp import MCPServerStdio

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BINARY = ROOT / "target/release/codex-tools-mcp"


def ensure_binary(binary: Path | None) -> Path:
    if binary.exists() is not None:
        return binary

    raise SystemExit(
        "Could not locate codex-tools-mcp binary at target/release/codex-tools-mcp. "
        "Build it with `cargo build --release` or pass --binary explicitly."
    )


async def run_agent(binary: Path, workdir: Path, prompt: str) -> None:
    if not binary.exists():
        raise SystemExit(
            f"MCP binary not found at {binary}. Build it with `cargo build --release`."
        )

    async with MCPServerStdio(
        name="codex-tools",
        params={
            "command": str(binary),
            "args": [],
            "cwd": str(workdir),
        },
        cache_tools_list=True,
    ) as server:
        agent = Agent(
            name="Assistant",
            instructions="Use the codex MCP tools to manage the plan and edit files.",
            mcp_servers=[server],
        )

        result = await Runner.run(agent, prompt)
        print(f"Agent output: {result.final_output}")

    target = workdir / "hello.txt"
    assert target.exists(), "hello.txt was not created"
    content = target.read_text()
    assert content.strip() == "hello world!", f"Unexpected content in hello.txt: {content!r}"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        type=Path,
        default=DEFAULT_BINARY,
        help="Path to the compiled codex-tools-mcp binary",
    )
    parser.add_argument(
        "--workdir",
        type=Path,
        default=ROOT,
        help="Working directory for the MCP server (defaults to repository root)",
    )
    parser.add_argument(
        "--prompt",
        default="Put 'hello world!' into hello.txt. Use planning to solve the task.",
        help="Prompt to send to the agent",
    )
    args = parser.parse_args()

    binary_path = ensure_binary(args.binary.resolve() if args.binary else None).resolve()
    asyncio.run(run_agent(binary_path, args.workdir.resolve(), args.prompt))


if __name__ == "__main__":
    main()
