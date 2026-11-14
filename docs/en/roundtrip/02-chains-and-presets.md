**English** · [简体中文](../../zh-cn/roundtrip/02-chains-and-presets.md)

# Round trip — Chains and presets

[← Domain overview](OVERVIEW.md) · [Documentation index](../README.md)

A round trip is a list of hops. Machine translation changes the words and keeps
the sentence structure. A chain picks the languages and, optionally, the
engines; presets are chains that were measured and named.

## How a chain works

A hop is `[engine:]lang`. A chain is a comma-separated list of hops:

```bash
translatex roundtrip "<text>" --from en --chain "google:zh,edge:ja,google:en"
```

- No engine in a hop: the automatic provider chain serves it (up to `race_width` providers race).
- A named engine: that engine is preferred for the hop (see [preferred engines](#preferred-engines-and-fallback)).
- The result must end in the source language. When the last hop is not the source, a final hop back to it is appended.

Shortcuts that expand to a chain:

| Flag | Expands to |
| --- | --- |
| `--depth 1` | one hop out through zh, then back |
| `--depth 2` | zh, ja, back |
| `--depth 3` | zh, ja, fi, back |
| `--pivot ja` | one hop out through ja, then back |
| `--preset <name>` | a named chain from the table below |

Chinese sources use en instead of zh: depth 3 runs en, ja, fi, then back to Chinese.
A source that is already on the path skips it, so a Japanese source runs zh, fi.

Precedence, first match wins: `--chain`, `--preset`, `--depth`, `--pivot`, then
`roundtrip_preset` from the config (default `light`).

## Presets

| Preset | Hops (English source) | Typical latency |
| --- | --- | --- |
| `light` (default) | zh, ja, en | about 1 s |
| `standard` | zh, ja, fi, en | about 1 s |
| `strong` | google to zh, edge to ja, google to fi, edge back to en | about 1 s |

`strong` takes the `standard` path and alternates two engines, so neighbouring
hops come from different models. Any other preset name fails with
`INVALID_INPUT`, listing `light, standard, strong`. `--depth 1-3` and `--chain`
build other chains by hand.

Set the default in `config.toml`:

```toml
roundtrip_preset = "standard"  # default: light
```

Environment: `TRANSLATEX_ROUNDTRIP_PRESET`.

## Preserving code and links

Protected spans are masked before each hop and restored afterwards; a hop that
drops or duplicates a placeholder fails and the engine moves on. Name patterns
instead of writing regexes: `--preserve @code` (inline and fenced code) and
`--preserve @url`. On Windows the npm `.cmd` shim runs through cmd.exe, which
strips `^` from a regex such as `` `[^`]*` `` when Python or Node spawns
translatex without a shell. The named patterns avoid that.

## Fact guard

Every round trip, whatever the chain, compares the result with the source and
lists in `missingFacts` the tokens that are absent:

- code spans, URLs, paths, ticket IDs, @mentions and numbers: always. Numbers compare by digits, so `18,900` matches `18900` and `4:50pm` matches `4:50 PM`.
- capitalised names, when both texts are in Latin script: as whole words, so `Tomi` does not vouch for `Tom`. Mid-sentence names are case-sensitive. A sentence-initial name is checked case-insensitively unless it is a common sentence-starting word or also appears in lowercase in the source, so a name that doubles as such a word can slip through.

A non-empty list also adds a `fact check: missing <list>` warning. The guard
reports; it does not fix. Read it, then restore the token by hand or re-run with
`--preserve`.

## Preferred engines and fallback

A named engine in a chain is preferred, not pinned. On a retryable error
(`RATE_LIMITED`, `TIMEOUT`, `NETWORK_ERROR`, `QUOTA_EXHAUSTED`) the hop falls
back to another engine from the automatic chain, avoiding the engines of the
neighbouring hops, and adds a warning:

```
hop 2: edge failed (RATE_LIMITED), served by google
```

## When to use which

| Situation | Choose |
| --- | --- |
| Check a translation by reading it back | `--preset light` (default) |
| More rewriting, a few more facts at risk | `--preset standard` or `--depth 3`; fact loss rises faster than rewriting |
| Different engines on neighbouring hops | `--preset strong` |
| Text with code or URLs | add `--preserve @code --preserve @url` to any preset |
| Legal text, contracts, numbers someone will check | do not round-trip |

Read `missingFacts` and `warnings` after every run, and report them.

## Navigation

- ← [Round trip overview](OVERVIEW.md)
- → [Choosing the pivot and depth](01-pivot-choice.md)
- ↑ [Documentation index](../README.md)
