"""Browser CDP connection utilities for Zymatica Agent Harness."""

from __future__ import annotations
import urllib.request

DEFAULT_BROWSER_CDP_URL = "http://localhost:9222"

def is_browser_debug_ready(url: str = DEFAULT_BROWSER_CDP_URL) -> bool:
    try:
        with urllib.request.urlopen(f"{url.rstrip('/')}/json/version", timeout=1):
            return True
    except Exception:
        return False

def manual_chrome_debug_command(port: int = 9222, system: str = "") -> str:
    return f'google-chrome --remote-debugging-port={port} --user-data-dir="/tmp/chrome-debug"'

def try_launch_chrome_debug(port: int = 9222, system: str = "") -> bool:
    return is_browser_debug_ready(f"http://localhost:{port}")
