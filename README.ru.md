<div align="center">

# translatex

**Бесплатный перевод для ИИ-агентов и скриптов: без API-ключа, без регистрации, без настройки.**

Движки, которым ключ не нужен, работают параллельно, первый подходящий ответ приходит в JSON, а типизированные ошибки подсказывают агенту следующий шаг. Один статический бинарник около 6 МБ и один файл skill, который 24 кодовых агента ставят копированием. Когда нужен SLA, добавьте ключи DeepL, Azure, Google Cloud и других.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#документация-и-лицензия)
[![Tests](https://img.shields.io/badge/tests-264-brightgreen.svg)](#разработка)
[![Providers](https://img.shields.io/badge/providers-24-blueviolet.svg)](#провайдеры)
[![Binary](https://img.shields.io/badge/binary-6%20MB-lightgrey.svg)](#установка)
[![Agent-native](https://img.shields.io/badge/agent--native-JSON-purple.svg)](#поддерживаемые-harness)

`translatex translate "Hello world" -t zh`

[English](./README.md) · [简体中文](./README.zh-cn.md) · [Русский](./README.ru.md) · [Документация](./docs/README.md)

</div>

---

## Содержание

- [Что вы получаете](#что-вы-получаете)
- [Установка](#установка)
- [Быстрый старт](#быстрый-старт)
- [Почему translatex](#почему-translatex)
- [Поддерживаемые harness](#поддерживаемые-harness)
- [Обратный перевод](#обратный-перевод)
- [Провайдеры](#провайдеры)
- [Статус каналов](#статус-каналов-на-2026-11-03)
- [Конфигурация](#конфигурация)
- [CLI](#cli)
- [Ошибки](#ошибки)
- [Документация и лицензия](#документация-и-лицензия)
- [Разработка](#разработка)

## Что вы получаете

- **Движки без ключа работают параллельно.** `translatex translate` запускает до трёх бесплатных движков одновременно (Google, Edge, Papago, Sogou и другие). Побеждает первый подходящий ответ, остальные отменяются, поэтому зависший эндпоинт не задерживает запрос.
- **Вывод в JSON.** Структурированный JSON или NDJSON в stdout, логи в stderr, коды возврата, по которым агент выбирает действие, и типизированные ошибки с `retryable` и `suggestedAction`.
- **Код и ссылки остаются на месте.** `--preserve @code` и `--preserve @url` (или любое регулярное выражение) маскируют фрагменты перед переводом и восстанавливают после. Если восстановление испорчено, попытка считается неудачной, и плохой результат не уходит вызывающему.
- **Пакетный NDJSON.** `translatex batch` читает по одному запросу на строку, выполняет их параллельно и возвращает по одному результату на строку в порядке ввода. Подходит для i18n-файлов и локализации.
- **Один бинарник.** Около 6 МБ, без рантайма, Docker и базы данных.
- **Файл skill для кодовых агентов.** Claude Code, Codex, Cursor, Zed и ещё 20 harness читают один skill или один блок инструкций. Без MCP-сервера, процесса и порта.
- **Ключи, когда они нужны.** DeepL, Azure, Google Cloud, Baidu, Tencent и ещё четыре официальных API подключаются к цепочке при наличии учётных данных. Ключи ротируются и остывают сами.
- **Обратный машинный перевод.** `translatex roundtrip` переводит текст через другие языки и обратно и перечисляет потерянные имена, числа и код.

## Установка

```bash
npm install -g translatex                 # готовый бинарник для вашей платформы
cargo install translatex                  # или сборка из исходников, Rust 1.89+
# или готовый бинарник:
# https://github.com/w21180239/translatex/releases/latest
```

Проверка:

```bash
translatex --version
translatex providers --format text
```

## Быстрый старт

```bash
# Без настройки, только бесплатные движки
translatex translate "Hello world" -t zh

# Закрепить провайдера и оставить резерв
translatex translate "Deploy failed" -t ja --provider deepl --fallback google,edge

# Чтение из stdin
echo "Good morning" | translatex translate -t fr

# Указать исходный язык вместо автоопределения
translatex translate "Guten Tag" -s de -t en

# Пакетная обработка NDJSON, по четыре запроса
translatex batch --file examples/batch.ndjson --parallel 4

# Не переводить код в обратных кавычках
translatex translate 'Run `make test` before pushing.' -t zh --preserve @code

# Прочитать перевод обратно через машинный перевод и посмотреть missingFacts
translatex roundtrip "Pay NZD 18,900 by Friday." --from en
```

Успешный ответ:

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

Ошибка несёт машиночитаемую причину:

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

## Почему translatex

| Почему выбирают | Что это значит |
| --- | --- |
| **Бесплатно, без пробного периода** | 14 публичных эндпоинтов (Google Translate, Bing/Edge, Papago, Sogou и другие) отвечают без ключа, аккаунта и таблицы квот. |
| **Ноль настройки** | Установили и переводите. Конфигурация нужна, чтобы закрепить провайдера или добавить ключи. |
| **Быстро по построению** | До трёх провайдеров работают наперегонки, побеждает первый подходящий ответ. |
| **Для агентов** | JSON или NDJSON в stdout, логи в stderr, коды возврата, типизированные ошибки и `manifest` со списком всех операций. |
| **Без MCP-сервера** | Интеграция это CLI и один документ skill. Хостить нечего, синхронизировать нечего. |
| **Растёт вместе с ключами** | Добавьте официальных провайдеров, когда нужен SLA. Ключи ротируются и остывают между процессами. |
| **Ничего не надо хостить** | Один статический бинарник. Ноутбук, CI-раннер или контейнер. |

### Как устроена цепочка

```
запрос ─▶ приоритет 1-12  бесплатные публичные эндпоинты  (без ключей и регистрации)
       ─▶ приоритет 20-27 официальные API                  (только при наличии учётных данных)
```

Чем меньше число, тем раньше попытка. Каждая попытка записывается, поэтому при сбое видно, какие движки опрошены и почему каждый отказал.

## Поддерживаемые harness

Интеграция состоит из команды и одного документа, который говорит агенту, когда её запускать: skill там, где harness загружает skills, файл правил там, где читают правила, блок инструкций там, где держат инструкции проекта. Сервер запускать не нужно.

<div align="center">
<table>
<tr>
<td align="center" width="150"><img src="repo-assets/agents/claude-code.png" width="44" alt="Claude Code"><br/><b>Claude Code</b><br/><sub>skill · плагин</sub></td>
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
<td align="center"><img src="repo-assets/agents/goose.png" width="44" alt="Goose"><br/><b>Goose</b><br/><sub>инструкции</sub></td>
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
<td align="center"><img src="repo-assets/agents/qwenpaw.svg" width="44" alt="QwenPaw"><br/><b>QwenPaw</b><br/><sub>инструкции</sub></td>
</tr>
</table>

<i>24 harness, два документа. Любой другой хост просто вызывает CLI.</i>
</div>

```bash
# Хосты со skills
mkdir -p .claude/skills/translatex && cp integrations/skill/translatex/SKILL.md .claude/skills/translatex/
mkdir -p ~/.config/opencode/skills/translatex && cp integrations/skill/translatex/SKILL.md ~/.config/opencode/skills/translatex/
mkdir -p ~/.agents/skills/translatex && cp integrations/skill/translatex/SKILL.md ~/.agents/skills/translatex/   # Codex, Gemini CLI

# Остальные читают тот же текст под своим именем файла
cat integrations/guidance/AGENTS.md >> AGENTS.md
```

| Что нужно | Где смотреть |
| --- | --- |
| Точный путь и команда для каждого harness | [integrations/README.md](./integrations/README.md) |
| Четыре канала и проверка установки | [docs/en/integrations/02-supported-harnesses.md](./docs/en/integrations/02-supported-harnesses.md) |
| Установка плагина Claude Code одной командой | [integrations/claude-code/README.md](./integrations/claude-code/README.md) |

Агенты, которым удобнее справочник команд, могут вызывать CLI напрямую:

| Команда | Назначение |
| --- | --- |
| `translatex translate "…" -t zh --format json` | Один перевод |
| `translatex roundtrip "…" --from en --format json` | Через шаги машинного перевода и обратно |
| `translatex providers --format json` | Что доступно на этой машине |
| `translatex health --format json` | Проверка всех настроенных провайдеров |

## Обратный перевод

`roundtrip` переводит текст через один или несколько шагов машинного перевода и возвращает его на исходный язык. Язык с другим синтаксисом заставляет перестроить текст, поэтому на обратном пути формулировки получаются новые. Режим подходит, чтобы прочитать перевод обратно и увидеть, что уцелело, или чтобы получить пересказ только средствами движков. LLM и ключи не нужны.

```bash
translatex roundtrip "<text>" --from en --preserve @code --format json
translatex roundtrip "<text>" --from en --preset standard             # zh, ja, fi и обратно
translatex roundtrip "<text>" --from en --depth 3                     # тот же путь, что у standard
translatex roundtrip "<text>" --from en --chain "google:zh,edge:ja,google:en"
translatex roundtrip --from en --pivot ja < draft.md
```

- **Выбор шагов.** Побеждает первое совпадение: `--chain "<[engine:]lang,...>"`, `--preset <name>`, `--depth 1-3`, `--pivot <lang>`, затем `roundtrip_preset` из конфигурации (по умолчанию `light`). Если цепочка не возвращается на исходный язык, шаг назад добавляется сам.
- **Пресеты.** `light` идёт через zh и ja и обратно (два промежуточных языка, по умолчанию). `standard` добавляет fi (три промежуточных языка). `strong` идёт по пути `standard`, чередуя google и edge, так что соседние шаги выполняют разные движки.
- **Проверка фактов.** `missingFacts` перечисляет имена, числа, ID, фрагменты кода и URL исходника, которых нет в результате (числа сравниваются по цифрам, имена по целым словам), и добавляется предупреждение `fact check:`. Имя, совпадающее с частым словом начала предложения, может проскочить.
- **Предпочитаемые движки.** Названный движок предпочитается, но не закрепляется. При повторяемой ошибке (лимит, таймаут, сеть, квота) шаг переходит на другой движок, избегая движков соседних шагов, и добавляет предупреждение.
- **Плейсхолдеры.** На каждом шаге защищённые фрагменты маскируются. Используйте `--preserve @code` и `--preserve @url`, а не регулярное выражение вроде `` `[^`]*` ``: в Windows npm-обёртка `.cmd` работает через cmd.exe, который вырезает `^`, если Python или Node запускает translatex без shell.

Замер 2026-11-11 (10 текстов, по одному запуску): один шаг потерял 5 фактов, два шага по умолчанию потеряли 5, три шага потеряли 8 при небольшом приросте переписывания, поэтому по умолчанию два шага. Машинный перевод меняет слова и сохраняет структуру предложений, а даже один шаг может исказить имя (Aroha в Aloha). Метод и цифры: [docs/en/roundtrip/01-pivot-choice.md](./docs/en/roundtrip/01-pivot-choice.md).

Поля ответа: `text`, `detectedSource`, `chain` (пройденные языки), `hops` (`from`, `to`, `provider`, `text`, `latencyMs`), `provider` (последний шаг), `missingFacts`, `warnings`, `latencyMs`, `requestId`, `chars`.

Не пропускайте через обратный перевод юридические тексты, справочники API, конфигурации и числа, которые читатель будет проверять: каждый шаг пересказывает, а проверка фактов эвристическая. Цепочки и пресеты: [docs/en/roundtrip/02-chains-and-presets.md](./docs/en/roundtrip/02-chains-and-presets.md).

## Провайдеры

Всего 24 провайдера: 15 веб-эндпоинтов без ключа и 9 официальных API с вашим ключом. Приоритет это число, по которому сортируется цепочка. `Символы` это лимит запроса, который адаптер соблюдает до разбиения.

### Веб-эндпоинты без ключа, основной путь

| # | Провайдер | ID | Автоопределение | Символы | Примечание |
| --- | --- | --- | --- | --- | --- |
| 1 | Google Translate (web) | `google` | да | 5 000 | `translate_a/single`, best effort |
| 2 | Microsoft Edge Translator | `edge` | да | 50 000 | без ключа |
| 3 | Iciba | `iciba` | да | 3 000 | подпись AES-ECB |
| 4 | iFlytek iFlyrec | `iflyrec` | да | 2 000 | только пары с китайским |
| 4 | ModernMT | `modernmt` | да | 5 000 | md5 + timestamp |
| 5 | Volcengine | `volcengine` | да | 5 000 | `crx/translate/v1/` |
| 5 | Lara Translate | `lara` | да | 500 | дневной лимит на IP |
| 6 | Lingvanex | `lingvanex` | нет | 1 000 | `API_BEARER_TOKEN` |
| 7 | Mirai Translate | `mirai` | да | 500 | только пары с японским, пробный ключ на каждый вызов |
| 8 | Papago (Naver) | `papago` | да | 1 000 | cookie-primed detect + form POST |
| 9 | Reverso Context | `reverso` | нет | 2 000 | трёхбуквенные коды; не входит в авто-цепочку |
| 10 | Sogou | `sogou` | да | 5 000 | md5-подпись; лимит на IP даёт `RATE_LIMITED` |
| 11 | MyMemory | `mymemory` | нет | 5 000 | коды RFC 3066, дневной лимит |
| 12 | SysTran | `systran` | нет | 5 000 | OAuth client credentials из бандла |
| — | LibreTranslate | `libretranslate` | да | 5 000 | только свой сервер: `[provider.libretranslate] base_url` |

### Официальные API, при наличии учётных данных

| # | Провайдер | ID | Учётные данные |
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


## Статус каналов на 2026-11-03

| Статус | Провайдеры |
| --- | --- |
| Работают полностью | `google`, `edge`, `iciba`, `modernmt`, `lara`, `volcengine`, `lingvanex`, `papago`, `sogou`, `mymemory`, `systran` |
| Только часть пар | `iflyrec`, `mirai` |
| Троттлинг под нагрузкой | `sogou` — `RATE_LIMITED` с `retryAfterSecs` |
| Блокировка по отпечатку TLS | `reverso` — Cloudflare отклоняет rustls JA3; остаётся доступным явно, но пропускается авто-цепочкой |
| Только свой сервер | `libretranslate` |
| Выведен вышестоящим | `iflytek` — псевдоним `iflyrec` |

Проверяйте на своей сети: `translatex health --format json`.


## Конфигурация

`~/.config/translatex/config.toml`:

```toml
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

[deepl.backup]
api_key = "..."
```

Порядок: ярлыки из `credentials.toml` идут в порядке файла, переменная `TRANSLATEX_<PROVIDER>_<FIELD>` добавляет ярлык `env` после них, а поле со значением `@keyring` читается из системного keyring. Бесплатным эндпоинтам ничего из этого не нужно.

## CLI

| Команда | Назначение |
| --- | --- |
| `translate [TEXT]` | Один запрос, при отсутствии текста читает stdin |
| `roundtrip [TEXT]` | Через шаги машинного перевода и обратно (`--chain`, `--preset`, `--depth`, `--pivot`) |
| `batch` | NDJSON на входе и выходе, порядок сохраняется |
| `providers` | Возможности и состояние конфигурации |
| `languages` | Языковые коды |
| `keys [PROVIDER]` | Состояние cooldown, `--reset` для сброса |
| `health` | Проверка всех настроенных провайдеров |
| `schema` | JSON Schema (draft 2020-12) |
| `manifest` | Машиночитаемый список операций CLI |
| `init` | Шаблон конфигурации |
| `completions SHELL` | Автодополнение |

Коды возврата: `0` успех, `1` ошибка запроса, `2` ошибка конфигурации, `3` цепочка исчерпана, `4` сетевая ошибка.

## Ошибки

| `code` | Повтор | Что делать агенту |
| --- | --- | --- |
| `RATE_LIMITED` | да | Подождать `retryAfterSecs` и повторить |
| `QUOTA_EXHAUSTED` | нет | Сменить ключ или дождаться квоты |
| `NETWORK_ERROR` | да | Экспоненциальная пауза |
| `UPSTREAM_ERROR` | только 5xx | Повтор на 5xx, исправить вход на 4xx |
| `TIMEOUT` | да | Увеличить `timeoutMs` или сменить провайдера |
| `AUTH_FAILED` | нет | Обновить учётные данные |
| `PROVIDER_NOT_CONFIGURED` | нет | Добавить ключ |
| `PROVIDER_NOT_FOUND` | нет | Исправить имя провайдера |
| `AUTO_DETECT_UNSUPPORTED` | нет | Передать `sourceLang` |
| `LANGUAGE_PAIR_NOT_SUPPORTED` | нет | Сменить провайдера или язык |
| `INVALID_INPUT` | нет | Исправить запрос |
| `CHAIN_EXHAUSTED` | нет | Смотреть `attempts` в `--verbose` |

Полный справочник: [docs/en/api/OVERVIEW.md](./docs/en/api/OVERVIEW.md).


## Документация и лицензия

- [docs/en/OVERVIEW.md](./docs/en/OVERVIEW.md): английская документация
- [docs/zh-cn/OVERVIEW.md](./docs/zh-cn/OVERVIEW.md): китайская документация
- [docs/en/roundtrip/OVERVIEW.md](./docs/en/roundtrip/OVERVIEW.md): обратный перевод, цепочки, пресеты, замеры
- [integrations/README.md](./integrations/README.md): подключение к 24 harness
- [AGENTS.md](./AGENTS.md): интеграция с агентами

Чтобы текст, написанный ИИ на английском, читался как написанный человеком, см. https://github.com/w21180239/not-a-bot.

## Разработка

```bash
cargo test                # 264 теста
cargo build --release     # около 5 минут с LTO
cargo clippy --all-targets
```

Новый провайдер: реализуйте `Provider` в `src/providers/<name>.rs`, задайте `priority` 1-12 для веб-эндпоинтов или 20-27 для API, зарегистрируйте в `src/cli.rs::build_registry` и добавьте имя в `KNOWN_PROVIDERS`.

## Лицензия

MIT OR Apache-2.0

Используйте на выбор [Apache License 2.0](LICENSE-APACHE) или [MIT](LICENSE-MIT).
