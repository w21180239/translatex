**简体中文** · [English](../../en/roundtrip/OVERVIEW.md)

# 往返翻译 — 概览

[← 文档索引](../README.md)

`roundtrip` 把文本经一跳或多跳机器翻译送出去，再翻回源语言：

```
原文 ──▶ 第 1 跳 ──▶ 第 2 跳 ──▶ … ──▶ 源语言
```

翻进句法差异很大的语言，就无法沿用原来的措辞，所以翻回来会得到新的说法。可以用它读回译文、看哪些内容留了下来，也可以只靠翻译引擎得到一份改写。人名、数字和代码会与原文比对，丢失时列在 `missingFacts` 里。全程不涉及 LLM，也不需要 API Key。

## 与翻译的先后顺序

`roundtrip` 返回的语言就是出发时的语言，所以目标语言不同时先翻译：

```bash
translatex translate "<text>" -t zh --format json               # 译成中文
translatex roundtrip "<中文文本>" --from zh --format json        # 改写这段中文
```

## 什么时候会出问题

往返会改写措辞，所以对精确内容会跑偏：

| 内容 | 适合往返吗？ |
| --- | --- |
| 营销文案、公告、博客文章 | 适合 |
| 带少量人名和数字的文章 | 适合，并读 `missingFacts` |
| 法律文本、合同、政策 | 不适合 |
| API 参考、配置示例、错误信息 | 不适合 |
| 读者会核对的数字、日期、人名、版本号 | 不适合 |
| 代码、标识符、路径 | 绝不；用 `--preserve @code` 放行 |

事实检查会列出结果丢了什么，但不能让往返对需要核对的文本变得安全。响应的 `hops` 带有每一跳的输出，中间结果可以审计。

## 接口

```bash
translatex roundtrip "Pay NZD 18,900 by Friday." --from en --format json
translatex roundtrip "…" --from en --preset standard      # zh、ja、fi，再回来
translatex roundtrip "…" --from en --preset strong        # 同一路径，google 与 edge 交替
translatex roundtrip "…" --from en --depth 3              # zh -> ja -> fi -> en
translatex roundtrip "…" --from en --chain "google:zh,edge:ja,google:en"
translatex roundtrip --from en --pivot ja < draft.md
translatex roundtrip "Run `make test`" --from en --preserve @code
```

选跳规则，先匹配者生效：`--chain`、`--preset`、`--depth`、`--pivot`，最后是配置里的 `roundtrip_preset`（默认 `light`）。

| 参数 | 含义 |
| --- | --- |
| `--chain "<[engine:]lang,...>"` | 显式指定跳；末尾没回到源语言时自动补一跳 |
| `--preset <name>` | `light`（默认）、`standard`、`strong` |
| `--depth 1-3` | 依次经 zh、ja、fi（中文原文为 en、ja、fi） |
| `--pivot <lang>` | 单个中转语言，或 `auto` |
| `--preserve <pattern>` | 正则，或具名模式 `@code`、`@url` |

链与预设见 [02-chains-and-presets.md](02-chains-and-presets.md)。

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

| 字段 | 含义 |
| --- | --- |
| `text` | 最终结果，已回到源语言 |
| `detectedSource` | 第一跳检测到的语言 |
| `chain` | 源语言之后依次经过的语言 |
| `hops` | 每一跳：`from`、`to`、`provider`、输出 `text`、`latencyMs` |
| `provider` | 处理最后一跳的 Provider |
| `missingFacts` | 输入里有、输出里没有的 token（人名、数字、ID、URL、代码） |
| `warnings` | 非空时才出现：`fact check:`、`preserve:` 和引擎回退提示 |
| `latencyMs`、`requestId`、`chars` | 总耗时、关联 id、输入长度 |

## 中转语言的选择

`pivot: "auto"` 对非中文输入用中文，对中文输入用英文，是默认 `light` 预设的第一跳。[01-pivot-choice.md](01-pivot-choice.md) 给出了这一选择背后的测量，以及日语更合适的场合。

可以用 `--pivot` / `"pivot"` 按请求覆盖，也可以在 `config.toml` 里用 `roundtrip_pivot` 对所有请求生效。

## 成本

一次往返每跳消耗一次 Provider 调用：默认 `light` 是三次（`auto` 下检测出中文输入时更多，见 [01-pivot-choice.md](01-pivot-choice.md#auto-中转的重试逻辑)）。延迟大致是各跳之和，而不是最大值。

## 导航

- ← [文档索引](../README.md)
- → [中转语言与跳数的选择](01-pivot-choice.md)
- → [链与预设](02-chains-and-presets.md)
