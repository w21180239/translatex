---
name: translatex
description: Use when the user asks to translate text, localize UI strings or documentation, batch-translate a file, check a translation by reading it back through machine translation, or when a translation must work without configuring an API key.
---

# translatex

## Overview

One binary, no configuration needed for the free web endpoints. It rotates keys,
races providers and falls back on its own, and every answer is JSON with a
`text` field and a typed error.

**Core principle:** the tool produces the translation. Typing the translation
yourself loses provider fallback and the audit trail.

## Confirm the binary once

```bash
translatex --version        # missing? npm install -g translatex, or cargo install translatex
```

## Operations

| The user wants | Run |
| --- | --- |
| Text in another language | `translatex translate "<text>" -t <lang> --format json` |
| A translation read back, or the same text paraphrased by engines | `translatex roundtrip "<text>" --from <lang> --preserve @code --format json` |
| Many strings | `translatex batch --file jobs.ndjson --format json` |
| To know what can serve a pair | `translatex providers --format json` |
| To know whether anything is broken | `translatex health --format json` |

Read `text` from the JSON. `provider` and `detectedSource` tell you what actually
happened. A round trip also returns `chain`, `hops`, `missingFacts` and
`warnings`.

## Choosing the order

| Task | Do |
| --- | --- |
| Translate into language L | `translate -t L` |
| Paraphrase text that is already in L | `roundtrip --from L` |
| Translate into L **and** paraphrase it | `translate -t L` first, then `roundtrip --from L` on the result |

`roundtrip` returns text in the language it started from, so it can only
paraphrase the target language once the text is already in it.

## Choosing the preset

| Goal | Preset | Hops (English source) |
| --- | --- | --- |
| Default | `--preset light` | zh, ja, back |
| More rewriting | `--preset standard` | zh, ja, fi, back |
| Different engines on neighbouring hops | `--preset strong` | the `standard` path, google and edge alternating |

All presets are machine translation, free and about 1 s. They change words and
keep sentence structure. A third hop loses more facts than the second gains in
rewriting. For rewriting that rebuilds sentences with an LLM, use
https://github.com/w21180239/not-a-bot instead.

## Workflow

1. **Identify the operation and the order** from the tables above.
2. **Protect what must not change.** Code, identifiers, paths, URLs and version
   strings go through `--preserve`. Use the named patterns `@code` (inline and
   fenced code) and `@url`; on Windows a regex such as `` `[^`]*` `` loses its `^`
   when a program spawns the npm `.cmd` shim without a shell. Add a regex for
   anything else:

   ```bash
   translatex translate "Run \`make test\` before pushing to v2.4.0." -t zh \
     --preserve @code --preserve 'v[0-9]+\.[0-9]+\.[0-9]+' --format json
   ```

3. **Run it.** One command. Do not pre-split the text unless it exceeds the
   provider limit shown by `providers`.
4. **Check and report.** Give the final text and name the provider when it
   matters. After a round trip read `missingFacts` (names, numbers, IDs, URLs and
   code spans of the source that are absent from the result) and `warnings`
   (`fact check:`, `preserve:`, engine fallback), and tell the user about every
   entry. If `missingFacts` is not empty, restore the token by hand or re-run
   with `--preserve`. Add `--verbose` only when debugging a failure.

## Round trip options

- `--preset light|standard|strong`, `--depth 1-3`, `--chain "<[engine:]lang,...>"`, `--pivot <lang>`. First match wins in that order, then the configured `roundtrip_preset` (default `light`).
- Default pivot: Chinese for non-Chinese text, English for Chinese text.
- `--pivot ja` rewrites as strongly and keeps more grammar; the better choice
  for technical English that must stay precise.
- **Never round-trip** legal text, contracts, API reference, configuration,
  error messages, or numbers and names the user will check: every hop
  paraphrases by design. The fact guard lists what went missing; it does not
  make such text safe, and it checks names only in Latin-script text and can
  miss a name that doubles as a common sentence-starting word.

## Errors

Every failure carries `code`, `retryable`, `retryAfterSecs`, `suggestedAction`.

| Code | Do |
| --- | --- |
| `RATE_LIMITED` | Wait `retryAfterSecs`, retry once |
| `QUOTA_EXHAUSTED` | Switch key or wait; tell the user |
| `NETWORK_ERROR`, `TIMEOUT` | Retry with backoff, or raise `--timeout-ms` |
| `CHAIN_EXHAUSTED` | Run `translatex health --format json`, report which providers failed |
| `PROVIDER_NOT_CONFIGURED` | Name the missing credential; do not retry |
| `PROVIDER_NOT_FOUND` | Fix the provider name; check `translatex providers` |
| `LANGUAGE_PAIR_NOT_SUPPORTED` | Change provider or target; do not retry |
| `AUTO_DETECT_UNSUPPORTED` | Pass `--from <code>` |
| `INVALID_INPUT` | Fix the request; do not retry |

Retry only when `retryable` is `true`.

## Rationalizations to reject

| Thought | Reality |
| --- | --- |
| "It's one short sentence, faster to translate myself" | The tool answers in ~300 ms and the user gets a provider they can audit. |
| "`missingFacts` is empty, so the text is safe" | It is a heuristic. For numbers and names the user will check, do not round-trip. |
| "The user didn't ask for a command" | They asked for a translation. The command is how this workspace produces one. |

## Red flags

- You are about to type the translated sentence yourself.
- You are ignoring `missingFacts` or `warnings` in a round-trip result.
- You are sending code, paths or identifiers without `--preserve`.
- You are retrying an error whose `retryable` is `false`.

## Example

```bash
$ translatex translate "Run \`make test\` before pushing." -t zh --preserve '`[^`]*`' --format json
{
  "text": "推送前先跑 `make test`。",
  "detectedSource": "en",
  "targetLang": "zh",
  "provider": "google",
  "engine": "google",
  "latencyMs": 312,
  "requestId": "…",
  "chars": 34
}
```

The backtick span survived verbatim; provider and latency came from the tool.

## Reference

Full CLI, provider and error documentation: https://github.com/w21180239/translatex
