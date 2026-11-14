**English** · [简体中文](../../zh-cn/roundtrip/OVERVIEW.md)

# Round trip — Overview

[← Documentation index](../README.md)

`roundtrip` translates text out through one or more machine-translation hops and
back into the source language:

```
source text ──▶ hop 1 ──▶ hop 2 ──▶ … ──▶ source language
```

A translation into a language with different syntax cannot reuse the original
phrasing, so translating back produces new wording. Use it to read a
translation back and see what survived, or to get a paraphrase from engines
alone. Names, numbers and code are checked against the source and listed in
`missingFacts` when they went missing. No LLM and no API key are involved.

## Ordering with translation

`roundtrip` returns text in the language it started from, so translate first when
the target differs:

```bash
translatex translate "<text>" -t zh --format json               # into Chinese
translatex roundtrip "<Chinese text>" --from zh --format json   # paraphrase the Chinese
```

## When it hurts

The round trip paraphrases, so it can drift on anything precise:

| Content | Use round trip? |
| --- | --- |
| Marketing copy, announcements, blog prose | yes |
| Prose with a few names and numbers | yes, and read `missingFacts` |
| Legal text, contracts, policy | no |
| API reference, config examples, error text | no |
| Numbers, dates, names, versions the reader will check | no |
| Code, identifiers, paths | never; pass them through `--preserve @code` |

The fact guard lists what the result lost; it does not make a round trip safe for
text the reader will verify. The response carries every hop's output in `hops`,
so the intermediates can be audited.

## Interface

```bash
translatex roundtrip "Pay NZD 18,900 by Friday." --from en --format json
translatex roundtrip "…" --from en --preset standard      # zh, ja, fi, back
translatex roundtrip "…" --from en --preset strong        # same path, google and edge alternating
translatex roundtrip "…" --from en --depth 3              # zh -> ja -> fi -> en
translatex roundtrip "…" --from en --chain "google:zh,edge:ja,google:en"
translatex roundtrip --from en --pivot ja < draft.md
translatex roundtrip "Run `make test`" --from en --preserve @code
```

Hop selection, first match wins: `--chain`, `--preset`, `--depth`, `--pivot`, then
the configured `roundtrip_preset` (default `light`).

| Flag | Meaning |
| --- | --- |
| `--chain "<[engine:]lang,...>"` | Explicit hops; a final hop back to the source is appended when missing |
| `--preset <name>` | `light` (default), `standard`, `strong` |
| `--depth 1-3` | Hops through zh, ja, fi (Chinese sources: en, ja, fi) |
| `--pivot <lang>` | One bridge language, or `auto` |
| `--preserve <pattern>` | Regex, or the named patterns `@code` and `@url` |

Chains and presets are described in [02-chains-and-presets.md](02-chains-and-presets.md).

```json
{
  "text": "…",
  "detectedSource": "en",
  "chain": ["zh", "en"],
  "hops": [
    {"from": "en", "to": "zh", "provider": "google", "text": "……", "latencyMs": 310},
    {"from": "zh", "to": "en", "provider": "edge", "text": "…", "latencyMs": 280}
  ],
  "provider": "edge",
  "missingFacts": [],
  "latencyMs": 590,
  "requestId": "…",
  "chars": 297
}
```

| Field | Meaning |
| --- | --- |
| `text` | Final result, back in the source language |
| `detectedSource` | Language the first hop detected |
| `chain` | Languages visited after the source |
| `hops` | Per hop: `from`, `to`, `provider`, output `text`, `latencyMs` |
| `provider` | Provider that served the final hop |
| `missingFacts` | Input tokens (names, numbers, IDs, URLs, code) absent from the output |
| `warnings` | Present when non-empty: `fact check:`, `preserve:` and engine fallback notes |
| `latencyMs`, `requestId`, `chars` | Total wall clock, correlation id, input length |

## Pivot selection

`pivot: "auto"` uses Chinese for non-Chinese input and English for Chinese
input, and is the first hop of the default `light` preset.
[01-pivot-choice.md](01-pivot-choice.md) has the measurements behind that
choice and the cases where Japanese is better.

Override per request with `--pivot` / `"pivot"`, or for every request with
`roundtrip_pivot` in `config.toml`.

## Cost

One round trip costs one provider call per hop: three for the default `light`
(more for Chinese input detected under `auto`, see
[01-pivot-choice.md](01-pivot-choice.md#the-auto-pivot-retry)). Latency is
roughly the sum of the hops, not the maximum.

## Navigation

- ← [Documentation index](../README.md)
- → [Choosing the pivot and the depth](01-pivot-choice.md)
- → [Chains and presets](02-chains-and-presets.md)
