"""
Fallback configuration for Zymatica CLI model providers.
"""

from typing import List, Dict, Any, Optional

def get_fallback_chain(primary_model: Optional[str] = None) -> List[Dict[str, Any]]:
    """Returns the ordered fallback chain for LLM model invocation."""
    return [
        {"provider": "zymatica-inference", "model": primary_model or "zymatica-vlm-cuneiform"},
        {"provider": "anthropic", "model": "claude-3-5-sonnet-20241022"},
        {"provider": "deepinfra", "model": "Qwen/Qwen2.5-72B-Instruct"},
        {"provider": "openrouter", "model": "meta-llama/llama-3.3-70b-instruct"},
    ]
