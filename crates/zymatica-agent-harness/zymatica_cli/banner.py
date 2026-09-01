"""
Zymatica CLI Banner Module
Handles version labeling, context length formatting, and welcome banner rendering.
"""

import os
import shutil
from typing import Optional, Any

from zymatica_cli import __version__

__release_date__ = "2026-08-31"


def _format_context_length(ctx_len: Optional[int]) -> str:
    """Format context length for display."""
    if ctx_len is None:
        return "N/A"
    if ctx_len >= 1_000_000:
        return f"{ctx_len / 1_000_000:.1f}M tokens"
    if ctx_len >= 1_000:
        return f"{ctx_len / 1_000:.1f}k tokens"
    return f"{ctx_len} tokens"


def format_banner_version_label() -> str:
    """Return formatted version string for banner."""
    return f"Zymatica Agent v{__version__} ({__release_date__})"


def build_welcome_banner(
    console: Any,
    model: Optional[str] = None,
    cwd: Optional[str] = None,
    tools: Optional[list] = None,
    enabled_toolsets: Optional[Any] = None,
    session_id: Optional[str] = None,
    context_length: Optional[int] = None,
    provider: Optional[str] = None,
) -> None:
    """Build and print the rich welcome banner for Zymatica CLI."""
    w = min(shutil.get_terminal_size().columns - 2, 88)
    if w < 30:
        if console:
            console.print(f"[bold gold1]⚕ NOUS ZYMATICA[/] [dim]- v{__version__}[/]")
        return

    banner_title = "⚕ NOUS ZYMATICA - AI Agent Framework"
    version_line = format_banner_version_label()
    model_line = f"Model: {model or 'default'} ({provider or 'local'}) | Context: {_format_context_length(context_length)}"
    cwd_line = f"Workspace: {cwd or os.getcwd()}"

    if console and hasattr(console, "print"):
        from rich.panel import Panel
        from rich.table import Table

        table = Table.grid(padding=(0, 1))
        table.add_row(f"[bold #FFBF00]{banner_title}[/]")
        table.add_row(f"[dim #B8860B]{version_line}[/]")
        table.add_row(f"[cyan]{model_line}[/]")
        table.add_row(f"[dim]{cwd_line}[/]")

        panel = Panel(
            table,
            border_style="#FFD700",
            expand=False,
            title="[bold #FFD700]Zymatica Shell[/]",
            title_align="left",
        )
        console.print(panel)
    else:
        print(f"=== {banner_title} ===")
        print(f"{version_line}")
        print(f"{model_line}")
        print(f"{cwd_line}")
