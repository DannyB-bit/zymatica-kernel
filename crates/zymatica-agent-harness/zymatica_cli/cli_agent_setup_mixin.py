"""
Agent setup and initialization mixin for Zymatica CLI.
"""

from typing import Any, Dict, List, Optional

class CLIAgentSetupMixin:
    """Provides agent initialization, tool registration, and skill loading capabilities."""
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.active_toolsets: List[str] = []
        self.active_skills: List[str] = []

    def setup_agent(self, toolsets: Optional[List[str]] = None, skills: Optional[List[str]] = None) -> None:
        self.active_toolsets = toolsets or ["core", "terminal", "cuneiform"]
        self.active_skills = skills or []
