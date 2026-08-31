"""
Zymatica CLI Package
Core interactive CLI, agent runtime, and configuration module for Zymatica Agent.
"""

from pathlib import Path
import sys

# Ensure agent harness root is on python path
_HARNESS_ROOT = Path(__file__).resolve().parent.parent
if str(_HARNESS_ROOT) not in sys.path:
    sys.path.insert(0, str(_HARNESS_ROOT))

__version__ = "1.0.0"
