**简体中文** · [English](../../en/roundtrip/02-chains-and-presets.md)

# 往返翻译 — 链与预设

[← 领域概览](OVERVIEW.md) · [文档索引](../README.md)

往返翻译就是一串"跳"。机器翻译改词、保留句子结构。链决定经过哪些语言，也可以指定引擎；预设是实测过并起了名字的链。

## 链如何工作

一跳写作 `[engine:]lang`。链是逗号分隔的跳列表：

```bash
translatex roundtrip "<text>" --from en --chain "google:zh,edge:ja,google:en"
```

- 跳里没有引擎：由自动 Provider 链处理（最多 `race_width` 个 Provider 并行竞速）。
- 点名引擎：该跳优先用这个引擎（见[优先引擎与回退](#优先引擎与回退)）。
- 结果必须回到源语言。最后一跳不是源语言时，会自动补一跳回去。

会展开成链的快捷参数：

| 参数 | 展开为 |
| --- | --- |
| `--depth 1` | 经 zh 出去一跳，再回来 |
| `--depth 2` | zh、ja，再回来 |
| `--depth 3` | zh、ja、fi，再回来 |
| `--pivot ja` | 经 ja 出去一跳，再回来 |
| `--preset <name>` | 下表里的命名链 |

中文原文用 en 代替 zh：深度 3 依次经 en、ja、fi，再回到中文。路径上已有的语言会被跳过，所以日语原文走 zh、fi。

选跳规则，先匹配者生效：`--chain`、`--preset`、`--depth`、`--pivot`，最后是配置里的 `roundtrip_preset`（默认 `light`）。

## 预设

| 预设 | 跳（英文原文） | 典型延迟 |
| --- | --- | --- |
| `light`（默认） | zh、ja、en | 约 1 s |
| `standard` | zh、ja、fi、en | 约 1 s |
| `strong` | google 到 zh，edge 到 ja，google 到 fi，edge 回 en | 约 1 s |

`strong` 走 `standard` 的路径，并交替使用两个引擎，相邻两跳来自不同的模型。其他预设名会返回 `INVALID_INPUT`，并列出 `light, standard, strong`。`--depth 1-3` 和 `--chain` 可以手工构造其它链。

在 `config.toml` 里设默认值：

```toml
roundtrip_preset = "standard"  # 默认：light
```

环境变量：`TRANSLATEX_ROUNDTRIP_PRESET`。

## 保护代码和链接

受保护的片段在每一跳前被遮蔽、之后还原；某一跳丢掉或重复占位符会让该次尝试失败，引擎换下一个。用具名模式代替手写正则：`--preserve @code`（行内和围栏代码）和 `--preserve @url`。在 Windows 上，npm 的 `.cmd` shim 经由 cmd.exe 运行，Python 或 Node 不经 shell 启动 translatex 时，会吃掉正则 `` `[^`]*` `` 里的 `^`。具名模式可以避开这个问题。

## 事实检查

每次往返，不论哪种链，都会把结果与原文比较，并在 `missingFacts` 里列出缺失的 token：

- 代码段、URL、路径、工单 ID、@提及和数字：始终检查。数字按数位比较，所以 `18,900` 匹配 `18900`，`4:50pm` 匹配 `4:50 PM`。
- 大写开头的人名，仅当两段文本都是拉丁字母时：按整词比较，所以 `Tomi` 不能替 `Tom` 作证。句中人名区分大小写。句首人名不区分大小写，除非它本身是常见的句首词，或在原文里也以小写出现，因此兼作这类词的人名可能漏检。

列表非空时还会加一条 `fact check: missing <list>` 警告。检查只报告，不修复。读完后手工补回，或带 `--preserve` 重跑。

## 优先引擎与回退

链里点名的引擎是"优先"而不是"锁定"。遇到可重试错误（`RATE_LIMITED`、`TIMEOUT`、`NETWORK_ERROR`、`QUOTA_EXHAUSTED`）时，该跳会改用自动链里的另一个引擎，避开相邻跳用过的引擎，并加一条警告：

```
hop 2: edge failed (RATE_LIMITED), served by google
```

## 怎么选

| 场景 | 选择 |
| --- | --- |
| 通过读回来核对译文 | `--preset light`（默认） |
| 改写更多，多冒一点事实风险 | `--preset standard` 或 `--depth 3`；事实丢失比改写增长得更快 |
| 相邻跳用不同引擎 | `--preset strong` |
| 含代码或 URL 的文本 | 任何预设都加 `--preserve @code --preserve @url` |
| 法律文本、合同、有人会核对的数字 | 不要往返 |

每次运行后都要读 `missingFacts` 和 `warnings`，并如实报告。

## 导航

- ← [往返翻译概览](OVERVIEW.md)
- → [中转语言与跳数的选择](01-pivot-choice.md)
- ↑ [文档索引](../README.md)
