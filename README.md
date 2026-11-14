<div align="center">

# translatex

**Free translation for AI agents and scripts: no API key, no signup, no config.**

Keyless translation engines are raced in parallel, the first good answer comes back as JSON, and typed errors tell your agent what to do next. One static binary of about 6 MB, plus one skill file that 24 coding agents can install with a copy. Add keys for DeepL, Azure, Google Cloud and others when you want an SLA.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Tests](https://img.shields.io/badge/tests-264-brightgreen.svg)](#development)
[![Providers](https://img.shields.io/badge/providers-24-blueviolet.svg)](#provider-matrix)
[![Binary](https://img.shields.io/badge/binary-6%20MB-lightgrey.svg)](#install)
[![Agent-native](https://img.shields.io/badge/agent--native-JSON-purple.svg)](#supported-harnesses)

`translatex translate "Hello world" -t zh`

[English](./README.md) · [简体中文](./README.zh-cn.md) · [Русский](./README.ru.md) · [Docs](./docs/README.md)

</div>

---

## Contents

- [What you get](#what-you-get)
- [Install](#install)
- [Quick start](#quick-start)
- [Why translatex](#why-translatex)
- [Supported harnesses](#supported-harnesses)
- [Round trip](#round-trip)
- [Provider matrix](#provider-matrix)
- [Live channel status](#live-channel-status)
- [Configuration](#configuration)
- [CLI reference](#cli-reference)
- [Error contract](#error-contract)
- [Documentation](#documentation)
- [Development](#development)

## What you get

- **Keyless engines, raced in parallel.** `translatex translate` runs up to three free engines at once (Google, Edge, Papago, Sogou and more). The first good answer wins and the rest are cancelled, so a hung endpoint cannot stall the request.
- **JSON out.** Structured JSON or NDJSON on stdout, logs on stderr, exit codes an agent can branch on, and typed errors with `retryable` and `suggestedAction`.
- **Code and links survive.** `--preserve @code` and `--preserve @url` (or any regex) mask spans before translation and restore them afterwards. A corrupted restore fails the attempt instead of shipping.
- **Batch NDJSON.** `translatex batch` takes one request per line, runs them in parallel and returns one result per line in input order. Good for i18n files and localisation jobs.
- **One binary.** About 6 MB, no runtime, no Docker, no database.
- **A skill file for coding agents.** Claude Code, Codex, Cursor, Zed and 20 more harnesses read one skill or one instructions block. No MCP server, no process, no port.
- **Keys when you want them.** DeepL, Azure, Google Cloud, Baidu, Tencent and four more join the chain when credentials exist. Keys rotate and cool down on their own.
- **A machine-translation round trip.** `translatex roundtrip` translates out through other languages and back, then lists any name, number or code that went missing.

## Install

```bash
npm install -g translatex                 # prebuilt binary for this platform
cargo install translatex                  # or build from source, Rust 1.89+
# or grab a prebuilt binary
# https://github.com/w21180239/translatex/releases/latest
```

Verify:

```bash
translatex --version
translatex providers --format text
```

## Quick start

```bash
# Keyless engines only, no configuration at all
translatex translate "Hello world" -t zh

# Pin a provider and keep a fallback list
translatex translate "Deploy failed" -t ja --provider deepl --fallback google,edge

# Read stdin
echo "Good morning" | translatex translate -t fr

# Pass the source language instead of detecting it
translatex translate "Guten Tag" -s de -t en

# NDJSON batch, four at a time
translatex batch --file examples/batch.ndjson --parallel 4

# Keep code spans intact
translatex translate 'Run `make test` before pushing.' -t zh --preserve @code

# Read a translation back through machine translation, then check missingFacts
translatex roundtrip "Pay NZD 18,900 by Friday." --from en
```

Successful response:

```json
{
  "text": "你好，世界",
  "detectedSource": "en",
  "targetLang": "zh",
  "provider": "google",
  "engine": "google",
  "latencyMs": 412,
  "requestId": "c506d3bb-8eb2-451a-9e84-f2be63833f79",
  "chars": 11
}
```

Failures carry a machine-readable reason:

```json
{
  "error": {
    "code": "PROVIDER_NOT_CONFIGURED",
    "message": "provider deepl is implemented but not configured; add credentials or a base_url",
    "retryable": false,
    "suggestedAction": "configure_a_provider"
  },
  "requestId": ""
}
```

## Why translatex

| Why it gets picked | What that means |
| --- | --- |
| **Free, with no trial period** | 14 public endpoints (Google Translate, Bing/Edge, Papago, Sogou and more) answer without a key, an account or a quota sheet. |
| **Zero config** | Install and translate. Config files exist for pinning providers and adding keys. |
| **Fast by construction** | Up to three providers race; the first good answer wins. |
| **Agent-native** | JSON or NDJSON on stdout, logs on stderr, exit codes, typed errors, and a `manifest` that lists every operation. |
| **No MCP server** | The integration is the CLI plus one skill document. Nothing to host and nothing to keep in sync. |
| **Grows with your keys** | Add keyed providers when you need an SLA; keys rotate and cool down across processes. |
| **Nothing to host** | One static binary. Laptop, CI runner or container. |

### How the chain works

```
request ─▶ priority 1-12  free public endpoints  (no keys, no signup)
        ─▶ priority 20-27 official APIs          (only if credentials exist)
```

Lower number means tried earlier. Every attempt is recorded, so a failure names the engines that were consulted and why each one declined.

## Supported harnesses

The integration is the command plus one document that tells the agent when to
run it: a skill where the harness loads skills, a rule file where it reads rules,
an instructions block where it keeps project instructions. No server, no protocol.

<div align="center">
<table>
<tr>
<td align="center" width="150"><img src="repo-assets/agents/claude-code.png" width="44" alt="Claude Code"><br/><b>Claude Code</b><br/><sub>skill · plugin</sub></td>
<td align="center" width="150"><img src="repo-assets/agents/codex.png" width="44" alt="Codex CLI"><br/><b>Codex CLI</b><br/><sub>skill · AGENTS.md</sub></td>
<td align="center" width="150"><img src="repo-assets/agents/gemini-cli.png" width="44" alt="Gemini CLI"><br/><b>Gemini CLI</b><br/><sub>skill · GEMINI.md</sub></td>
<td align="center" width="150"><img src="repo-assets/agents/opencode.png" width="44" alt="OpenCode"><br/><b>OpenCode</b><br/><sub>skill · AGENTS.md</sub></td>
</tr>
<tr>
<td align="center"><img src="repo-assets/agents/cursor.png" width="44" alt="Cursor"><br/><b>Cursor</b><br/><sub>.cursor/rules</sub></td>
<td align="center"><img src="repo-assets/agents/cline.png" width="44" alt="Cline"><br/><b>Cline</b><br/><sub>.clinerules</sub></td>
<td align="center"><img src="repo-assets/agents/roo-code.png" width="44" alt="Roo Code"><br/><b>Roo Code</b><br/><sub>.roo/rules</sub></td>
<td align="center"><img src="repo-assets/agents/kilo-code.png" width="44" alt="Kilo Code"><br/><b>Kilo Code</b><br/><sub>.kilocode/rules</sub></td>
</tr>
<tr>
<td align="center"><img src="repo-assets/agents/continue.png" width="44" alt="Continue"><br/><b>Continue</b><br/><sub>.continue/rules</sub></td>
<td align="center"><img src="repo-assets/agents/jetbrains.png" width="44" alt="JetBrains AI"><br/><b>JetBrains AI</b><br/><sub>.aiassistant/rules</sub></td>
<td align="center"><img src="repo-assets/agents/zed.png" width="44" alt="Zed"><br/><b>Zed</b><br/><sub>.rules</sub></td>
<td align="center"><img src="repo-assets/agents/aider.png" width="44" alt="Aider"><br/><b>Aider</b><br/><sub>CONVENTIONS.md</sub></td>
</tr>
<tr>
<td align="center"><img src="repo-assets/agents/qwen-code.png" width="44" alt="Qwen Code"><br/><b>Qwen Code</b><br/><sub>QWEN.md</sub></td>
<td align="center"><img src="repo-assets/agents/crush.png" width="44" alt="Crush"><br/><b>Crush</b><br/><sub>CRUSH.md</sub></td>
<td align="center"><img src="repo-assets/agents/goose.png" width="44" alt="Goose"><br/><b>Goose</b><br/><sub>instructions</sub></td>
<td align="center"><img src="repo-assets/agents/deepseek-harness.png" width="44" alt="DeepSeek Harness"><br/><b>DeepSeek Harness</b><br/><sub>AGENTS.md</sub></td>
</tr>
<tr>
<td align="center"><img src="repo-assets/agents/mimo-code.svg" width="44" alt="MiMo Code"><br/><b>MiMo Code</b><br/><sub>AGENTS.md</sub></td>
<td align="center"><img src="repo-assets/agents/atomcode.png" width="44" alt="AtomCode"><br/><b>AtomCode</b><br/><sub>AGENTS.md</sub></td>
<td align="center"><img src="repo-assets/agents/openclaw.png" width="44" alt="OpenClaw"><br/><b>OpenClaw</b><br/><sub>AGENTS.md</sub></td>
<td align="center"><img src="repo-assets/agents/hermes-agent.png" width="44" alt="Hermes Agent"><br/><b>Hermes Agent</b><br/><sub>AGENTS.md</sub></td>
</tr>
<tr>
<td align="center"><img src="repo-assets/agents/pi.png" width="44" alt="Pi"><br/><b>Pi</b><br/><sub>AGENTS.md</sub></td>
<td align="center"><img src="repo-assets/agents/reasonix.png" width="44" alt="Reasonix"><br/><b>Reasonix</b><br/><sub>AGENTS.md</sub></td>
<td align="center"><img src="repo-assets/agents/zcode.png" width="44" alt="ZCode"><br/><b>ZCode</b><br/><sub>AGENTS.md</sub></td>
<td align="center"><img src="repo-assets/agents/qwenpaw.svg" width="44" alt="QwenPaw"><br/><b>QwenPaw</b><br/><sub>instructions</sub></td>
</tr>
</table>

<i>24 harnesses, two documents. Anything else that can run a shell uses the CLI directly.</i>
</div>

```bash
# Skill harnesses
mkdir -p .claude/skills/translatex && cp integrations/skill/translatex/SKILL.md .claude/skills/translatex/
mkdir -p ~/.config/opencode/skills/translatex && cp integrations/skill/translatex/SKILL.md ~/.config/opencode/skills/translatex/
mkdir -p ~/.agents/skills/translatex && cp integrations/skill/translatex/SKILL.md ~/.agents/skills/translatex/   # Codex, Gemini CLI

# Everything else reads the same block under its own filename
cat integrations/guidance/AGENTS.md >> AGENTS.md
```

| Need | Read |
| --- | --- |
| Exact path and command per harness | [integrations/README.md](./integrations/README.md) |
| Why a skill beats an instructions file | [docs/en/integrations/02-supported-harnesses.md](./docs/en/integrations/02-supported-harnesses.md) |
| One-command plugin install for Claude Code | [integrations/claude-code/README.md](./integrations/claude-code/README.md) |

Agents that prefer a command reference over a skill can call the CLI directly:

| Command | Purpose |
| --- | --- |
| `translatex translate "…" -t zh --format json` | One translation |
| `translatex roundtrip "…" --from en --format json` | Out through machine-translation hops and back |
| `translatex providers --format json` | What can serve this machine |
| `translatex health --format json` | Probe every configured provider |

## Round trip

`roundtrip` translates text out through one or more machine-translation hops and back into the source language. A language with different syntax forces the text to be rebuilt, so the way back produces new wording. Use it to read a translation back and see what survived, or to get a paraphrase from engines alone. No LLM and no key are involved.

```bash
translatex roundtrip "<text>" --from en --preserve @code --format json
translatex roundtrip "<text>" --from en --preset standard             # zh, ja, fi, back
translatex roundtrip "<text>" --from en --depth 3                     # same path as standard
translatex roundtrip "<text>" --from en --chain "google:zh,edge:ja,google:en"
translatex roundtrip --from en --pivot ja < draft.md
```

- **Choosing hops.** First match wins: `--chain "<[engine:]lang,...>"`, `--preset <name>`, `--depth 1-3`, `--pivot <lang>`, then `roundtrip_preset` in the config (default `light`). A final hop back to the source is appended when a chain lacks one.
- **Presets.** `light` goes out through zh and ja and back (two pivots, the default). `standard` adds fi (three pivots). `strong` takes the `standard` path with google and edge alternating, so neighbouring hops come from different engines.
- **Fact guard.** `missingFacts` lists the names, numbers, IDs, code spans and URLs of the source that the result lacks (numbers compared by digits, names as whole words), and a `fact check:` warning is added. A name that doubles as a common sentence-starting word can slip through.
- **Preferred engines.** A named engine is preferred, not pinned. On a retryable error (rate limit, timeout, network, quota) the hop falls back to another engine, avoiding the neighbouring hops' engines, with a warning.
- **Placeholders.** Protected spans are masked on every hop. Use `--preserve @code` and `--preserve @url` rather than a regex such as `` `[^`]*` ``: on Windows the npm `.cmd` shim runs through cmd.exe, which strips `^` when Python or Node spawns translatex without a shell.

Measured on 2026-11-11 (10 texts, one run each): one hop lost 5 facts, the default two hops lost 5, and three hops lost 8 while rewriting only a little more, so the default is two hops. Machine translation changes words and keeps sentence structure, and even one hop can respell a name (Aroha to Aloha). Method and numbers: [docs/en/roundtrip/01-pivot-choice.md](./docs/en/roundtrip/01-pivot-choice.md).

Response fields: `text`, `detectedSource`, `chain` (languages visited), `hops` (`from`, `to`, `provider`, `text`, `latencyMs`), `provider` (final hop), `missingFacts`, `warnings`, `latencyMs`, `requestId`, `chars`.

Never round-trip legal text, API reference, configuration or numbers the reader will check; every hop paraphrases, and the fact guard is a heuristic. Chains and presets: [docs/en/roundtrip/02-chains-and-presets.md](./docs/en/roundtrip/02-chains-and-presets.md).

## Provider matrix

24 providers: 15 keyless web endpoints and 9 official APIs that take your key. Priorities are the numbers the chain sorts by. `chars` is the request limit the adapter enforces before splitting.

### Keyless web endpoints, the primary path

| # | Provider | ID | Auto-detect | Max chars | Notes |
| --- | --- | --- | --- | --- | --- |
| 1 | Google Translate (web) | `google` | yes | 5 000 | `translate.googleapis.com/translate_a/single`, best effort |
| 2 | Microsoft Edge Translator | `edge` | yes | 50 000 | `edge.microsoft.com/translate`, no key |
| 3 | Iciba | `iciba` | yes | 3 000 | AES-ECB signed payload |
| 4 | iFlytek iFlyrec | `iflyrec` | yes | 2 000 | zh-anchored pairs only |
| 4 | ModernMT | `modernmt` | yes | 5 000 | md5 verify + timestamp |
| 5 | Volcengine | `volcengine` | yes | 5 000 | `crx/translate/v1/` |
| 5 | Lara Translate | `lara` | yes | 500 | daily per-IP credit |
| 6 | Lingvanex | `lingvanex` | no | 1 000 | `API_BEARER_TOKEN` |
| 7 | Mirai Translate | `mirai` | yes | 500 | Japanese-anchored pairs only, trial key scraped per call |
| 8 | Papago (Naver) | `papago` | yes | 1 000 | cookie-primed detect + form POST |
| 9 | Reverso Context | `reverso` | no | 2 000 | three-letter codes; excluded from the auto chain, see status |
| 10 | Sogou | `sogou` | yes | 5 000 | md5 sign; per-IP throttle returns `RATE_LIMITED` |
| 11 | MyMemory | `mymemory` | no | 5 000 | RFC 3066 codes, daily cap |
| 12 | SysTran | `systran` | no | 5 000 | OAuth client credentials scraped from the bundle |
| - | LibreTranslate | `libretranslate` | yes | 5 000 | self-hosted only: set `[provider.libretranslate] base_url` |

### Official APIs, used when credentials exist

| # | Provider | ID | Credentials |
| --- | --- | --- | --- |
| 20 | DeepL | `deepl` | `api_key` |
| 20 | Baidu | `baidu` | `appid`, `secret_key` |
| 21 | Google Cloud Translation | `googlecloud` | `api_key` |
| 22 | Azure Translator | `azure` | `api_key`, `region` |
| 23 | Alibaba Cloud | `aliyun` | `access_key_id`, `access_key_secret` |
| 24 | Caiyun | `caiyun` | `token` |
| 25 | Niutrans | `niutrans` | `apikey` |
| 26 | Tencent | `tencent` | `secret_id`, `secret_key` |
| 27 | Youdao | `youdao` | `app_key`, `app_secret` |

## Live channel status

Endpoints drift. This is what a full sweep against the free web endpoints returned on 2026-11-03:

| Status | Providers |
| --- | --- |
| Verified end-to-end | `google`, `edge`, `iciba`, `modernmt`, `lara`, `volcengine`, `lingvanex`, `papago`, `sogou`, `mymemory`, `systran` |
| Restricted pairs | `iflyrec` (zh to any), `mirai` (ja-anchored) |
| Throttles under load | `sogou`: the per-IP limit surfaces as `RATE_LIMITED` with `retryAfterSecs`, never a fake translation |
| Blocked by fingerprinting | `reverso`: Cloudflare rejects the rustls JA3; it stays registered for explicit use but is skipped by the auto chain, so an automatic request never spends a round trip on it |
| Opt-in only | `libretranslate`: no public instance is alive; self-host and set `base_url` |
| Retired upstream | `iflytek`: aliased to `iflyrec` so old configs keep working |

Run `translatex health --format json` on your own network before trusting this table.

## Configuration

`~/.config/translatex/config.toml` (or `--config`, or `TRANSLATEX_CONFIG`):

```toml
# Replaces the built-in auto chain. Keyed providers are still appended
# afterwards if credentials exist.
chain = ["edge", "google", "iciba", "deepl"]

default_target = "zh"
timeout_ms = 20000
retries = 1
race_width = 3
roundtrip_pivot = "auto"
roundtrip_preset = "light"          # light, standard, strong
log = "warn"

[provider.libretranslate]
base_url = "http://localhost:5000"
```

`~/.config/translatex/credentials.toml`:

```toml
[deepl.default]
api_key = "..."

[azure.default]
api_key = "..."
region = "eastasia"

# A second key takes over when the first is cooling.
[deepl.backup]
api_key = "..."
```

Resolution: `credentials.toml` labels are tried in file order, a `TRANSLATEX_<PROVIDER>_<FIELD>` variable adds an `env` label after them, and a field set to `@keyring` is read from the OS keyring. Free endpoints need none of this.

## CLI reference

| Command | Purpose |
| --- | --- |
| `translate [TEXT]` | One request; reads stdin when text is omitted |
| `roundtrip [TEXT]` | Out through machine-translation hops and back (`--chain`, `--preset`, `--depth`, `--pivot`) |
| `batch` | NDJSON in, NDJSON out, order preserved |
| `providers` | Capability and configuration state per provider |
| `languages` | Canonical codes, or one provider's advertised set |
| `keys [PROVIDER]` | Cooldown and failure state; `--reset` clears it |
| `health` | Probe every configured provider |
| `schema` | JSON Schema (draft 2020-12) for all wire types |
| `manifest` | Machine-readable list of CLI operations |
| `init` | Write a config template |
| `completions SHELL` | Bash/zsh/fish/PowerShell completion |

Global flags: `--format json|text`, `--quiet`, `--verbose`, `--race N`, `--config`, `--credentials`, `--log`.

Exit codes: `0` success, `1` request error, `2` config error, `3` chain exhausted, `4` network error.

## Error contract

| `code` | Retryable | Agent action |
| --- | --- | --- |
| `RATE_LIMITED` | yes | Wait `retryAfterSecs`, then retry |
| `QUOTA_EXHAUSTED` | no | Rotate key or wait for the quota window |
| `NETWORK_ERROR` | yes | Exponential backoff |
| `UPSTREAM_ERROR` | 5xx only | Retry on 5xx, fix input on 4xx |
| `TIMEOUT` | yes | Raise `timeoutMs` and retry, or switch provider |
| `AUTH_FAILED` | no | Refresh credentials |
| `PROVIDER_NOT_CONFIGURED` | no | Add credentials |
| `PROVIDER_NOT_FOUND` | no | Fix the provider name |
| `AUTO_DETECT_UNSUPPORTED` | no | Pass an explicit `sourceLang` |
| `LANGUAGE_PAIR_NOT_SUPPORTED` | no | Change provider or target |
| `INVALID_INPUT` | no | Fix the request |
| `CHAIN_EXHAUSTED` | no | Inspect `--verbose` attempts |

Full reference: [docs/en/api/OVERVIEW.md](./docs/en/api/OVERVIEW.md).

## Documentation

| Language | Entry point |
| --- | --- |
| English | [docs/en/OVERVIEW.md](./docs/en/OVERVIEW.md) |
| 简体中文 | [docs/zh-cn/OVERVIEW.md](./docs/zh-cn/OVERVIEW.md) |
| Round trip: overview, chains and presets, measured results | [docs/en/roundtrip/OVERVIEW.md](./docs/en/roundtrip/OVERVIEW.md) |
| Harness setup | [integrations/README.md](./integrations/README.md) |
| Agent integration | [AGENTS.md](./AGENTS.md) |

## Development

```bash
cargo test                # 264 tests
cargo build --release     # ~5 min with LTO
cargo clippy --all-targets
```

Adding a provider: implement `Provider` in `src/providers/<name>.rs`, set `priority` in the 1-12 range for web endpoints or 20-27 for APIs, register it in `src/cli.rs::build_registry`, and add the name to `KNOWN_PROVIDERS`.

## Related

For making AI-written English read like a person wrote it, see https://github.com/w21180239/not-a-bot.

## License

MIT OR Apache-2.0

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
