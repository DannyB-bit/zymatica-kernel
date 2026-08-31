"""
Billing and usage tracking mixin for Zymatica CLI.
"""

from typing import Any, Dict, Optional

class CLIBillingMixin:
    """Provides billing, credit, and usage tracking hooks."""
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.session_cost: float = 0.0
        self.session_tokens_in: int = 0
        self.session_tokens_out: int = 0

    def record_usage(self, tokens_in: int, tokens_out: int, cost_usd: float = 0.0) -> None:
        self.session_tokens_in += tokens_in
        self.session_tokens_out += tokens_out
        self.session_cost += cost_usd

    def get_session_summary(self) -> Dict[str, Any]:
        return {
            "tokens_in": self.session_tokens_in,
            "tokens_out": self.session_tokens_out,
            "total_tokens": self.session_tokens_in + self.session_tokens_out,
            "cost_usd": self.session_cost,
        }
