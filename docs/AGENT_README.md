# Foundry Models as VS Code Coding Agents

Short guide to the Microsoft Foundry setup backing the AI coding agents in
this workspace. Verified 2026-11-22.

## Mental model

Three layers, often confused:

| Layer | What it is | Yours |
| --- | --- | --- |
| **Model** | Published weights in the Foundry catalog | `claude-opus-5` |
| **Deployment** | *Your* billable instance, with its own quota | `claude-opus-5` (same name) |
| **Agent** | Stateful Foundry service with instructions, tools, memory | **none — you have zero** |

A deployment is a phone number you call. An agent is an employee who
remembers prior calls. You are using **deployments only**.

**The coding agent is VS Code, not Foundry.** The agent loop — reading
files, running commands, retrying — runs locally. Foundry is stateless: it
receives the whole conversation each time and returns text or a tool call.
Your extension executes the tool. This is why long sessions drift toward
the context limit, and why Foundry portal settings do not affect VS Code.

## Current setup

Resource `gsanerds-resource` (eastus2), project `gsanerds`:

| Deployment | Capacity | Context | Notes |
| --- | --- | --- | --- |
| `claude-opus-5` | 5000 | 1,000,000 | Primary. Native tool calling. |
| `claude-fable-5-2` | 2000 | 1,000,000 | Different family, not an Opus version. |
| `gpt-6.1-sol` | 250 | — | Tools require the Responses API (below). |

Both Claude models have an **identical** 1M context window — measured, not
assumed. Clients advertise 872,000 as a safety margin so output and the
next tool result still fit. They differ in *quota*, not context.

```bash
az cognitiveservices account deployment list \
  -n gsanerds-resource -g development_group -o table
```

## Adding a model to VS Code

AI Toolkit (`ms-windows-ai-studio`) auto-discovers deployments. Nothing is
hand-edited.

1. Open the **AI Toolkit** panel
2. **My Models → Microsoft Foundry → Refresh**
3. Click the deployment to add it
4. Select it in the chat model picker

Registered IDs look like
`aitk-foundry/Microsoft Foundry/(gsanerds)claude-opus-5`, cached in
`globalStorage/state.vscdb` (not a settings file).

## Where each setting actually lives

This is the part that causes the most confusion.

| Setting | Where | In `settings.json`? |
| --- | --- | --- |
| Capacity / rate limits | Azure deployment (`az` CLI) | No |
| Context window | Property of the model | No, not configurable |
| Max output tokens | Sent per request by the client | No, auto-discovered |
| Thinking / effort | Sent per request by the client | No |
| AI Toolkit behavior | `windowsaistudio.*` keys | **Yes** |

**Foundry portal sliders (thinking, effort, max tokens) do not affect VS
Code.** They belong to the playground and to agent definitions. A
deployment stores no preferences — every request carries its own.

## Thinking / effort levels

These models use *adaptive* thinking. The older form is rejected:

```
"thinking.type.enabled" is not supported for this model.
Use "thinking.type.adaptive" and "output_config.effort"
```

Correct request shape:

```json
{
  "thinking":      { "type": "adaptive" },
  "output_config": { "effort": "low" }
}
```

Valid: `low`, `medium`, `high`, `xhigh`. **Thinking cannot be disabled**;
`low` is the floor. Measured cost on `claude-opus-5`:

| Effort | Thinking tokens | Total output |
| --- | --- | --- |
| low | 38 | 177 |
| medium | 90 | 262 |
| high | 103 | 318 |
| xhigh | 117 | 336 |

In clients: Copilot exposes `thinkingLevel` per model in
`chatLanguageModels.json`; AI Toolkit negotiates it automatically.

## Endpoints

Vendors are not interchangeable — the wrong route returns 404.

| Models | Endpoint |
| --- | --- |
| Claude | `/anthropic/v1/messages` (needs `anthropic-version` header) |
| GPT, plain chat | `/openai/v1/chat/completions` |
| GPT, with tools | `/openai/v1/responses` |

`gpt-6.1-sol` **cannot** use tools on `chat/completions`. It rejects them
alongside `reasoning_effort`, and the suggested workaround
(`reasoning_effort: "none"`) is not supported by the model — only `low`
through `xhigh`. Tool calling works solely via `/v1/responses`.

## Troubleshooting

**`422 ... empty response with no text or tool calls`**

A response is a list of blocks: `thinking`, `text`, `tool_use`. The client
needs `text` or `tool_use` to act on. Thinking tokens bill against
`max_tokens`, so a small budget plus high effort exhausts the budget during
reasoning and returns only an empty `thinking` block.

Reproduced with `max_tokens=32`:

```
stop_reason: max_tokens
output_tokens: 32, thinking_tokens: 32
blocks: [('thinking', 0)]   <- no text
```

Fixes, in order: raise max output tokens (128,000 verified accepted); lower
effort to `low`; start a fresh session if the context window is nearly full.

**Model missing from VS Code** — refresh AI Toolkit; the list is cached.

**Throttling** — check capacity. `gpt-6.1-sol` at 250 req/min may throttle
agentic work; Claude Opus at 5000 will not.

## Health check

```bash
# Claude
curl -s "https://gsanerds-resource.services.ai.azure.com/anthropic/v1/messages?api-version=2025-10-01-preview" \
  -H "Authorization: Bearer $(az account get-access-token \
      --resource https://cognitiveservices.azure.com --query accessToken -o tsv)" \
  -H "Content-Type: application/json" -H "anthropic-version: 2023-06-01" \
  -d '{"model":"claude-opus-5","max_tokens":64,"messages":[{"role":"user","content":"say OK"}]}'
```

A healthy reply ends with `"stop_reason":"end_turn"` and a non-empty
`text` block.

## Notes

- `claudeCode.*` settings were removed from user `settings.json` on
  2026-11-22: no installed extension read them and the `claude` CLI is not
  on PATH. Backup: `settings.json.bak-20261122-154513`.
- An AI Toolkit agent draft named `codehelp` exists locally but was never
  published, so it has no effect.
- The empty `gsanerds` project is required — deployments live under it.
