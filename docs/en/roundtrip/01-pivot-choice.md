**English** · [简体中文](../../zh-cn/roundtrip/01-pivot-choice.md)

# Round trip — Choosing the pivot and the depth

[← Domain overview](OVERVIEW.md) · [Documentation index](../README.md)

The first half of this page covers the pivot language (2026-11-04). The second
half covers how many hops (2026-11-11).

## Choosing the pivot

The pivot has to be far enough from the source language to force a rewrite, and
well enough supported that both hops succeed. Those pull in opposite directions,
so the choice was measured rather than guessed.

## Method

Five English sentences with stock openers, hedge phrases and
"not just X but Y" constructions were round-tripped through six pivots on
2026-11-04, using the default auto chain. For each pair:

- **Word overlap** — Jaccard similarity between content words of the input and
  the result, stop words removed. Lower means more rewriting.
- **Length delta** — mean absolute change in length.
- **Latency** — wall clock for both hops.

The 2026-11-04 script was not kept. For a quick check on one text, run:

```bash
translatex roundtrip "Our comprehensive solution leverages cutting-edge technology." --from en --pivot ja --format json
```

## Results

| Pivot | Word overlap | Length change | Latency | What happened |
| --- | --- | --- | --- | --- |
| `ru` | 0.561 | 6.9% | 634 ms | Most rewriting, but the most semantic drift |
| `ja` | 0.582 | 5.3% | 512 ms | Strong rewrite, keeps more structure than Chinese |
| `zh` | 0.618 | 7.1% | 600 ms | Strong rewrite, best provider coverage |
| `fr` | 0.675 | 2.2% | 586 ms | Mild; some sentences came back unchanged |
| `de` | 0.695 | 2.0% | 633 ms | Mild |
| `es` | 0.791 | 1.6% | 565 ms | Barely changes anything |

Representative outputs:

```
source  Our comprehensive solution leverages cutting-edge technology to empower
        teams to unlock their full potential and drive meaningful outcomes.
zh      Our integrated solutions leverage cutting-edge technology to empower
        teams to unlock their full potential and drive meaningful results.
ja      Our comprehensive solutions leverage cutting-edge technology to help
        teams maximize their potential and deliver meaningful results.
de      Our comprehensive solution leverages cutting-edge technology to empower
        teams to reach their full potential and achieve meaningful results.
```

German, French and Spanish preserve English sentence structure, so the round trip
returns almost the input. They are the wrong pivots when the aim is new wording.

## Recommendation

| Source | Default pivot | Why |
| --- | --- | --- |
| Anything non-Chinese | `zh` | Rewrites as much as Japanese, and is served by every provider including the Chinese-anchored ones |
| Chinese (`zh`, `zh-TW`, `yue`) | `en` | The mirror of the same rule |

`auto` implements exactly that table. It is the first hop of the default `light`
preset and the only hop of `--pivot`; `--depth` adds ja and fi after it.

**Japanese is the documented alternative.** It rewrites slightly more and keeps
more grammatical structure, which is an advantage for technical English where
Chinese drops articles and plurals. Set it when the text must stay precise:

```bash
translatex roundtrip api-notes.md --from en --pivot ja
```

**Russian rewrites most but drifts most.** It is not the default for a reason:
in the sample it replaced "comprehensive" with "end-to-end" and dropped
qualifiers. Use it only when maximal rewriting matters more than fidelity.

## The auto-pivot retry

With `sourceLang: "auto"` the language is unknown when the pivot must be chosen,
so `auto` guesses Chinese first and inspects `detectedSource` afterwards. If the
text turns out to be Chinese, the guessed pivot *was* the source language, and
the first hop is redone into English. That costs one extra call only for Chinese
input; every other language takes one call per hop.

## Configuring it

Per request:

```json
{ "text": "…", "pivot": "ja" }
```

Every request:

```toml
# ~/.config/translatex/config.toml
roundtrip_pivot = "zh"
```

## Choosing the depth

Does a second or third machine-translation hop rewrite more? Ten texts were
round-tripped per configuration on 2026-11-11: five English paragraphs, three
fact-heavy workplace texts (names, numbers, IDs, code) and two Chinese texts.

- **Overlap**: content-word Jaccard with the source. Lower means more rewriting.
- **Facts missing**: names, numbers, IDs and code spans the fact guard reports as absent, summed over the corpus.
- **Latency**: mean wall clock for the whole chain.

| Configuration | Overlap | Facts missing | Latency |
| --- | --- | --- | --- |
| 1 hop (`--depth 1`, 0.1.x default) | 0.556 | 5 | 0.5 s |
| 2 hops (`light`, the default) | 0.520 | 5 | 0.9 s |
| 3 hops (`--depth 3`, `standard`) | 0.480 | 8 | 1.1 s |

What the numbers say:

- Machine translation changes words and keeps sentence structure, so "not just X, it's Y" survives every depth.
- Even one hop respells names (Aroha became Aloha, Tom became Tomi) and expands codes (NZD became "New Zealand dollars").
- A third hop costs more facts (8 against 5) than it gains in rewriting (overlap 0.480 against 0.520). The default moved from one hop to two; three hops stay available as `--depth 3` and `standard`.

Caveats: ten texts and one run per configuration is a small sample; the fact
guard is a heuristic; overlap rewards change, not quality.

## Navigation

- ← [Round trip overview](OVERVIEW.md)
- → [Chains and presets](02-chains-and-presets.md)
- ↑ [Documentation index](../README.md)
