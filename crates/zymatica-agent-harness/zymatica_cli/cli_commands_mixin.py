"""
Interactive slash-command handler mixin for Zymatica CLI.
"""

from typing import Any, Callable, Dict, List, Optional

class CLICommandsMixin:
    """Provides command parsing and execution dispatch for interactive CLI sessions."""
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.command_registry: Dict[str, Callable[..., Any]] = {}

    def register_command(self, name: str, handler: Callable[..., Any]) -> None:
        self.command_registry[name.strip().lower()] = handler

    def handle_command(self, raw_input: str) -> bool:
        """Returns True if input was recognized and handled as a command, False otherwise."""
        if not raw_input.startswith("/"):
            return False
        parts = raw_input.strip()[1:].split(maxsplit=1)
        if not parts:
            return False
        cmd = parts[0].lower()
        arg = parts[1] if len(parts) > 1 else ""
        handler = self.command_registry.get(cmd)
        if handler:
            handler(arg)
            return True
        return False
