"""Environment loader for Zymatica Agent Harness."""

from __future__ import annotations
import os
from pathlib import Path

def load_zymatica_dotenv(zymatica_home: str | Path | None = None, project_env: str | Path | None = None) -> None:
    """Load environment variables from ~/.zymatica/.env or project .env."""
    paths = []
    if zymatica_home:
        paths.append(Path(zymatica_home) / ".env")
    if project_env:
        paths.append(Path(project_env))
        
    for p in paths:
        if p.is_file():
            try:
                with open(p, "r", encoding="utf-8", errors="ignore") as f:
                    for line in f:
                        line = line.strip()
                        if line and not line.startswith("#") and "=" in line:
                            k, v = line.split("=", 1)
                            k, v = k.strip(), v.strip().strip("'\"")
                            if k not in os.environ:
                                os.environ[k] = v
            except Exception:
                pass
