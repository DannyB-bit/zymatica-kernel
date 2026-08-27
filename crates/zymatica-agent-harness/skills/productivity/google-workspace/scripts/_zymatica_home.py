"""Resolve ZYMATICA_HOME for standalone skill scripts.

Skill scripts may run outside the Zymatica process (e.g. system Python,
nix env, CI) where ``zymatica_constants`` is not importable.  This module
provides the same ``get_zymatica_home()`` and ``display_zymatica_home()``
contracts as ``zymatica_constants`` without requiring it on ``sys.path``.

When ``zymatica_constants`` IS available it is used directly so that any
future enhancements (profile resolution, Docker detection, etc.) are
picked up automatically.  The fallback path replicates the core logic
from ``zymatica_constants.py`` using only the stdlib.

All scripts under ``google-workspace/scripts/`` should import from here
instead of duplicating the ``ZYMATICA_HOME = Path(os.getenv(...))`` pattern.
"""

from __future__ import annotations

import os
from pathlib import Path

try:
    from zymatica_constants import display_zymatica_home as display_zymatica_home
    from zymatica_constants import get_zymatica_home as get_zymatica_home
except (ModuleNotFoundError, ImportError):

    def get_zymatica_home() -> Path:
        """Return the Zymatica home directory (default: ~/.zymatica).

        Mirrors ``zymatica_constants.get_zymatica_home()``."""
        val = os.environ.get("ZYMATICA_HOME", "").strip()
        return Path(val) if val else Path.home() / ".zymatica"

    def display_zymatica_home() -> str:
        """Return a user-friendly ``~/``-shortened display string.

        Mirrors ``zymatica_constants.display_zymatica_home()``."""
        home = get_zymatica_home()
        try:
            return "~/" + str(home.relative_to(Path.home()))
        except ValueError:
            return str(home)
