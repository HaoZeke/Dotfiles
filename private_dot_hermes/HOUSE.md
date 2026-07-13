# Hermes house ownership (SURF / Willma)

## Files

| Path | Owner | Notes |
|------|--------|------|
| `~/.hermes/config.yaml` | **chezmoi** `private_dot_hermes/config.yaml` | Non-secret only. Model, terminal, MCP, agent knobs. |
| `~/.hermes/SOUL.md` | **chezmoi** `private_dot_hermes/SOUL.md` | House SURF persona. |
| `~/.hermes/.env` | **live + pass inject** | Secrets only (`OPENAI_API_KEY`, `OPENAI_BASE_URL`, …). Never git. |
| sessions/logs/state.db | Hermes runtime | Do not manage with chezmoi. |

## SURF model

- Primary: `openai/gpt-oss-120b` via custom OpenAI-compatible endpoint `https://willma.surf.nl/api/v0`
- Token source: `pass show surf/ai-hub/token` → `.env` `OPENAI_API_KEY` (see `run_onchange_*hermes*` script)
- Schema: `_config_version: 33+`; `model` is a mapping (`provider` / `default` / `base_url`)

## After `hermes model` / dashboard rewrites

Hermes may rewrite `config.yaml` (and put keys into it). Recover:

```bash
chezmoi apply ~/.hermes/config.yaml ~/.hermes/SOUL.md
# re-inject secrets
bash ~/.local/share/chezmoi/.chezmoiscripts/...  # or: chezmoi apply --force for the onchange script
```

Prefer `${OPENAI_API_KEY}` in config; keep literal keys only in `.env`.

## Optional providers

Doctor lists many optional keys (OpenRouter, Telegram, …). House policy: **do not** import every optional key. SURF/Willma is primary for SURF hosts.

## Config shape (reconciled 2026-07-13)

Source of truth is the **live Hermes CLI shape** (as on rgSURFLat after
`hermes model` / migrate), sealed back into chezmoi:

- `model.provider: custom:willma` (named custom provider; not bare `willma`)
- Willma URL + `default_model` under `providers.willma`
- `api_key: ${OPENAI_API_KEY}` only (secrets stay in `.env`)
- `hooks.transform_llm_output` → `agent-hooks/willma-advisor.sh`
- `platform_toolsets.cli` tool list (CLI surface)
- Same MCP set: eb-stack, cf-ci, context7, project-ctx, nimvault, ookcite

After a fresh `hermes model` rewrite: re-export live → `age` seal → push
chezmoi, or `chezmoi apply` only if you intend to reset to sealed.
