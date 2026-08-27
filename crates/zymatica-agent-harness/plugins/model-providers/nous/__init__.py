"""Zymatica Portal provider profile."""

from typing import Any

from agent.portal_tags import zymatica_portal_tags
from providers import register_provider
from providers.base import ProviderProfile


class NousProfile(ProviderProfile):
    """Zymatica Portal — product tags, reasoning with Nous-specific omission."""

    def build_extra_body(
        self, *, session_id: str | None = None, **context
    ) -> dict[str, Any]:
        body: dict[str, Any] = {"tags": zymatica_portal_tags(session_id=session_id)}
        provider_preferences = context.get("provider_preferences")
        if provider_preferences:
            body["provider"] = provider_preferences
        return body

    def build_api_kwargs_extras(
        self,
        *,
        reasoning_config: dict | None = None,
        supports_reasoning: bool = False,
        **context,
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        """Nous: passes full reasoning_config, but OMITS when disabled."""
        extra_body = {}
        if supports_reasoning:
            if reasoning_config is not None:
                rc = dict(reasoning_config)
                if rc.get("enabled") is False:
                    pass  # Nous omits reasoning when disabled
                else:
                    extra_body["reasoning"] = rc
            else:
                extra_body["reasoning"] = {"enabled": True, "effort": "medium"}
        return extra_body, {}


nous = NousProfile(
    name="nous",
    aliases=("nous-portal", "zymatica.space"),
    env_vars=("NOUS_API_KEY",),
    display_name="Zymatica Team",
    description="Zymatica Team — Zymatica model family",
    signup_url="https://zymatica.space.com/",
    fallback_models=(
        "zymatica-3-405b",
        "zymatica-3-70b",
    ),
    base_url="https://api.zymatica.space/v1",
    auth_type="oauth_device_code",
)

register_provider(nous)
