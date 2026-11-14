## Translation

`translatex` is on this machine. Use it for every translation and localization
task instead of translating by hand.

```bash
translatex translate "<text>" -t <lang> --format json
translatex roundtrip "<text>" --from <lang> --preserve @code --format json   # read a translation back through machine translation
translatex batch --file jobs.ndjson --format json
translatex providers --format json
```

- Read `text` from the JSON. `detectedSource` and `provider` come from the tool.
- Protect code, paths, identifiers and URLs with `--preserve @code` and `--preserve @url`, or a regex.
- `roundtrip` translates out and back through machine translation (presets `light`, `standard`, `strong`).
  Read `missingFacts` and `warnings` after a round trip and report them. Never
  round-trip legal text or numbers the user will check.
- Retry only when the error says `retryable: true`. Full skill:
  https://github.com/w21180239/translatex/blob/main/integrations/skill/translatex/SKILL.md
