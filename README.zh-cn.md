<div align="center">

# translatex

**给 AI Agent 和脚本用的免费翻译：不用 API Key，不用注册，不用配置。**

无需 Key 的翻译引擎并行竞速，第一个合格的答案以 JSON 返回，带类型的错误告诉 Agent 下一步该做什么。一个约 6 MB 的静态二进制，外加一份 skill 文件，24 个编程 Agent 复制一下就能装上。需要 SLA 时再补 DeepL、Azure、Google Cloud 等的 Key。

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#许可)
[![Tests](https://img.shields.io/badge/tests-264-brightgreen.svg)](#开发)
[![Providers](https://img.shields.io/badge/providers-24-blueviolet.svg)](#provider-一览)
[![Binary](https://img.shields.io/badge/binary-6%20MB-lightgrey.svg)](#安装)
[![Agent-native](https://img.shields.io/badge/agent--native-JSON-purple.svg)](#支持的-harness)

`translatex translate "Hello world" -t zh`

[English](./README.md) · [简体中文](./README.zh-cn.md) · [Русский](./README.ru.md) · [文档](./docs/README.md)

</div>

---

## 目录

- [你能得到什么](#你能得到什么)
- [安装](#安装)
- [快速开始](#快速开始)
- [为什么选 translatex](#为什么选-translatex)
- [支持的 harness](#支持的-harness)
- [往返翻译](#往返翻译)
- [Provider 一览](#provider-一览)
- [实测状态](#实测状态)
- [配置](#配置)
- [CLI 参考](#cli-参考)
- [错误契约](#错误契约)
- [文档](#文档)
- [开发](#开发)

## 你能得到什么

- **无需 Key 的引擎并行竞速。** `translatex translate` 最多同时跑三个免费引擎（Google、Edge、Papago、Sogou 等），第一个合格的答案胜出，其余取消，所以挂起的接口拖不住请求。
- **JSON 输出。** stdout 输出结构化 JSON 或 NDJSON，日志走 stderr，退出码可供 Agent 分支判断，错误带类型，并附 `retryable` 和 `suggestedAction`。
- **代码和链接原样保留。** `--preserve @code`、`--preserve @url`（或任意正则）在翻译前遮蔽片段，翻译后还原。还原出错会让该次尝试失败，而不是把坏结果交出去。
- **批量 NDJSON。** `translatex batch` 每行一个请求，并行执行，按输入顺序每行返回一个结果，适合 i18n 文件和本地化任务。
- **一个二进制。** 约 6 MB，无运行时，无 Docker，无数据库。
- **给编程 Agent 的 skill 文件。** Claude Code、Codex、Cursor、Zed 等 24 个 harness 读同一份 skill 或指令块。没有 MCP 服务，没有进程，没有端口。
- **需要时再加 Key。** DeepL、Azure、Google Cloud、百度、腾讯等九个官方 API 在有凭据时加入回退链，Key 自动轮换并冷却。
- **机器翻译往返。** `translatex roundtrip` 经其它语言翻出去再翻回来，并列出丢失的人名、数字和代码。

## 安装

```bash
npm install -g translatex                 # 当前平台的预编译二进制
cargo install translatex                  # 或从源码构建，Rust 1.89+
# 或下载预编译二进制
# https://github.com/w21180239/translatex/releases/latest
```

验证：

```bash
translatex --version
translatex providers --format text
```

## 快速开始

```bash
# 只用免费引擎，完全不用配置
translatex translate "Hello world" -t zh

# 指定 Provider 并保留回退列表
translatex translate "Deploy failed" -t ja --provider deepl --fallback google,edge

# 读 stdin
echo "Good morning" | translatex translate -t fr

# 手动指定源语言，不做检测
translatex translate "Guten Tag" -s de -t en

# NDJSON 批量，一次四个
translatex batch --file examples/batch.ndjson --parallel 4

# 保护代码片段
translatex translate 'Run `make test` before pushing.' -t zh --preserve @code

# 经机器翻译读回译文，再看 missingFacts
translatex roundtrip "Pay NZD 18,900 by Friday." --from en
```

成功响应：

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

失败时带机器可读的原因：

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

## 为什么选 translatex

| 为什么会被选 | 意味着什么 |
| --- | --- |
| **免费，没有试用期** | 14 个公共接口（Google 翻译、Bing/Edge、Papago、Sogou 等）不用 Key、不用账号、不看配额表。 |
| **零配置** | 装好就能翻译。配置文件只用来固定 Provider 和添加 Key。 |
| **天然快** | 最多三个 Provider 竞速，第一个合格的答案胜出。 |
| **Agent 原生** | stdout 输出 JSON 或 NDJSON，日志走 stderr，有退出码、带类型的错误，还有列出所有操作的 `manifest`。 |
| **没有 MCP 服务** | 集成就是 CLI 加一份 skill 文档。不用托管，也不用保持同步。 |
| **随 Key 扩展** | 需要 SLA 时加官方 Provider，Key 跨进程轮换和冷却。 |
| **无需托管** | 一个静态二进制。笔记本、CI 机器或容器都能跑。 |

### 回退链如何工作

```
请求 ─▶ 优先级 1-12  免费公共接口  （无需 Key，无需注册）
     ─▶ 优先级 20-27 官方 API      （仅在存在凭据时）
```

数字越小越先尝试。每次尝试都会被记录，所以失败时会指出问了哪些引擎、各自为什么拒绝。

## 支持的 harness

接入方式是一条命令加一份文档：宿主会加载 skill 就给 skill，读项目规则就给规则文件，读项目指令就追加一段。没有服务要启动，也没有协议要协商。

<div align="center">
<table>
<tr>
<td align="center" width="150"><img src="repo-assets/agents/claude-code.png" width="44" alt="Claude Code"><br/><b>Claude Code</b><br/><sub>skill · 插件</sub></td>
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
<td align="center"><img src="repo-assets/agents/goose.png" width="44" alt="Goose"><br/><b>Goose</b><br/><sub>指令</sub></td>
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
<td align="center"><img src="repo-assets/agents/qwenpaw.svg" width="44" alt="QwenPaw"><br/><b>QwenPaw</b><br/><sub>指令</sub></td>
</tr>
</table>

<i>24 家 harness，两份文档。其它只要能执行 shell，就直接走 CLI。</i>
</div>

```bash
# 支持 skill 的 harness
mkdir -p .claude/skills/translatex && cp integrations/skill/translatex/SKILL.md .claude/skills/translatex/
mkdir -p ~/.config/opencode/skills/translatex && cp integrations/skill/translatex/SKILL.md ~/.config/opencode/skills/translatex/
mkdir -p ~/.agents/skills/translatex && cp integrations/skill/translatex/SKILL.md ~/.agents/skills/translatex/   # Codex、Gemini CLI

# 其它 harness 读同一段内容，文件名各异
cat integrations/guidance/AGENTS.md >> AGENTS.md
```

| 需求 | 阅读 |
| --- | --- |
| 每个 harness 的确切路径和命令 | [integrations/README.md](./integrations/README.md) |
| 为什么 skill 比指令文件好 | [docs/zh-cn/integrations/02-supported-harnesses.md](./docs/zh-cn/integrations/02-supported-harnesses.md) |
| Claude Code 一条命令装插件 | [integrations/claude-code/README.md](./integrations/claude-code/README.md) |

偏好命令参考而非 skill 的 Agent，可以直接调用 CLI：

| 命令 | 用途 |
| --- | --- |
| `translatex translate "…" -t zh --format json` | 单条翻译 |
| `translatex roundtrip "…" --from en --format json` | 经多跳机器翻译翻出再翻回 |
| `translatex providers --format json` | 本机能用哪些 Provider |
| `translatex health --format json` | 探测所有已配置 Provider |

## 往返翻译

`roundtrip` 把文本经一跳或多跳机器翻译送出去，再翻回源语言。句法差异大的语言会迫使文本被重建，所以翻回来的措辞是新的。可以用它读回译文、看哪些内容留了下来，也可以只靠翻译引擎得到一份改写。不涉及 LLM，也不需要 Key。

```bash
translatex roundtrip "<text>" --from en --preserve @code --format json
translatex roundtrip "<text>" --from en --preset standard             # zh、ja、fi，再回来
translatex roundtrip "<text>" --from en --depth 3                     # 与 standard 同一路径
translatex roundtrip "<text>" --from en --chain "google:zh,edge:ja,google:en"
translatex roundtrip --from en --pivot ja < draft.md
```

- **选跳规则。** 先匹配者生效：`--chain "<[engine:]lang,...>"`、`--preset <name>`、`--depth 1-3`、`--pivot <lang>`，最后是配置里的 `roundtrip_preset`（默认 `light`）。链末尾没回到源语言时会自动补一跳。
- **预设。** `light` 经 zh、ja 出去再回来（两个中转，默认）。`standard` 再加 fi（三个中转）。`strong` 走 `standard` 的路径，google 与 edge 交替，相邻两跳来自不同引擎。
- **事实检查。** `missingFacts` 列出原文里有、结果里没有的人名、数字、ID、代码段和 URL（数字按数位比较，人名按整词比较），并加一条 `fact check:` 警告。兼作常见句首词的人名可能漏检。
- **优先引擎。** 点名的引擎是"优先"而不是"锁定"。遇到可重试错误（限流、超时、网络、配额）时，该跳会改用另一个引擎，避开相邻跳用过的引擎，并加一条警告。
- **占位符。** 每一跳都会遮蔽受保护的片段。请用 `--preserve @code` 和 `--preserve @url`，不要写 `` `[^`]*` `` 这样的正则：在 Windows 上，npm 的 `.cmd` shim 经由 cmd.exe 运行，Python 或 Node 不经 shell 启动 translatex 时会吃掉 `^`。

2026-11-11 实测（10 段文本，各跑一次）：一跳丢 5 个事实，默认的两跳丢 5 个，三跳丢 8 个而多改写的幅度很小，所以默认是两跳。机器翻译改词、保留句子结构，哪怕一跳也可能改写人名（Aroha 变成 Aloha）。方法和数据：[docs/zh-cn/roundtrip/01-pivot-choice.md](./docs/zh-cn/roundtrip/01-pivot-choice.md)。

响应字段：`text`、`detectedSource`、`chain`（经过的语言）、`hops`（`from`、`to`、`provider`、`text`、`latencyMs`）、`provider`（最后一跳）、`missingFacts`、`warnings`、`latencyMs`、`requestId`、`chars`。

不要对法律文本、API 参考、配置，或读者会核对的数字做往返；每一跳都会改写措辞，事实检查只是启发式。链与预设：[docs/zh-cn/roundtrip/02-chains-and-presets.md](./docs/zh-cn/roundtrip/02-chains-and-presets.md)。

## Provider 一览

共 24 个 Provider：15 个无需 Key 的网页接口，9 个使用你自己 Key 的官方 API。优先级即回退链排序依据。`字符数` 是适配器在切分前强制限制的单次请求上限。

### 无需 Key 的网页接口，主路径

| # | Provider | ID | 自动检测 | 字符数 | 备注 |
| --- | --- | --- | --- | --- | --- |
| 1 | Google 翻译（网页） | `google` | 是 | 5 000 | `translate.googleapis.com/translate_a/single`，尽力而为 |
| 2 | 微软 Edge 翻译 | `edge` | 是 | 50 000 | `edge.microsoft.com/translate`，无需 Key |
| 3 | 爱词霸 | `iciba` | 是 | 3 000 | AES-ECB 签名 |
| 4 | 讯飞 iflyrec | `iflyrec` | 是 | 2 000 | 仅支持中文对其它语言 |
| 4 | ModernMT | `modernmt` | 是 | 5 000 | md5 校验 + 时间戳 |
| 5 | 火山引擎 | `volcengine` | 是 | 5 000 | `crx/translate/v1/` |
| 5 | Lara Translate | `lara` | 是 | 500 | 按 IP 的每日额度 |
| 6 | Lingvanex | `lingvanex` | 否 | 1 000 | `API_BEARER_TOKEN` |
| 7 | Mirai Translate | `mirai` | 是 | 500 | 仅日文相关语言对，每次调用抓取 trial key |
| 8 | Papago（Naver） | `papago` | 是 | 1 000 | 先取 Cookie 再表单 POST |
| 9 | Reverso Context | `reverso` | 否 | 2 000 | 三字母语言码；不在自动链中，见状态表 |
| 10 | 搜狗翻译 | `sogou` | 是 | 5 000 | md5 签名；按 IP 限流时返回 `RATE_LIMITED` |
| 11 | MyMemory | `mymemory` | 否 | 5 000 | RFC 3066 语言码，有每日上限 |
| 12 | SysTran | `systran` | 否 | 5 000 | 从前端 bundle 抓 OAuth client credentials |
| — | LibreTranslate | `libretranslate` | 是 | 5 000 | 仅自建：配置 `[provider.libretranslate] base_url` |

### 官方 API，有凭据时使用

| # | Provider | ID | 凭据字段 |
| --- | --- | --- | --- |
| 20 | DeepL | `deepl` | `api_key` |
| 20 | 百度 | `baidu` | `appid`、`secret_key` |
| 21 | Google Cloud 翻译 | `googlecloud` | `api_key` |
| 22 | Azure 翻译 | `azure` | `api_key`、`region` |
| 23 | 阿里云 | `aliyun` | `access_key_id`、`access_key_secret` |
| 24 | 彩云小译 | `caiyun` | `token` |
| 25 | 小牛翻译 | `niutrans` | `apikey` |
| 26 | 腾讯云 | `tencent` | `secret_id`、`secret_key` |
| 27 | 有道 | `youdao` | `app_key`、`app_secret` |


## 实测状态

接口会变。以下是 2026-11-03 对免费网页接口做全量实测的结果：

| 状态 | Provider |
| --- | --- |
| 端到端可用 | `google`、`edge`、`iciba`、`modernmt`、`lara`、`volcengine`、`lingvanex`、`papago`、`sogou`、`mymemory`、`systran` |
| 语言对受限 | `iflyrec`（中文 ↔ 其它）、`mirai`（仅日文相关） |
| 高负载时限流 | `sogou` —— 按 IP 限流时报 `RATE_LIMITED` 并带 `retryAfterSecs`，绝不假装翻译成功 |
| 被指纹拦截 | `reverso` —— Cloudflare 拒绝 rustls 的 JA3；仍保留注册以便显式调用，但自动链会跳过它，自动请求不再浪费一次往返 |
| 仅自建 | `libretranslate` —— 已无可用公共实例，自建后配置 `base_url` |
| 上游停用 | `iflytek` —— 已别名到 `iflyrec`，旧配置继续可用 |

在你自己的网络环境里，先跑 `translatex health --format json` 再依赖这张表。


## 配置

`~/.config/translatex/config.toml`（也可用 `--config` 或 `TRANSLATEX_CONFIG`）：

```toml
# 替换内置 auto 链。有凭据的官方 Provider 仍会追加在后面。
chain = ["edge", "google", "iciba", "deepl"]

default_target = "zh"
timeout_ms = 20000
retries = 1
race_width = 3
roundtrip_pivot = "auto"
roundtrip_preset = "light"          # light、standard、strong
log = "warn"

[provider.libretranslate]
base_url = "http://localhost:5000"
```

`~/.config/translatex/credentials.toml`：

```toml
[deepl.default]
api_key = "..."

[azure.default]
api_key = "..."
region = "eastasia"

# 第一个 Key 冷却时由第二个接管。
[deepl.backup]
api_key = "..."
```

解析顺序：`credentials.toml` 里的 Key 标签按文件顺序轮换，`TRANSLATEX_<PROVIDER>_<FIELD>` 在其后追加一个 `env` 标签，字段写成 `@keyring` 时从系统钥匙串读取。免费接口三者都不需要。

## CLI 参考

| 命令 | 用途 |
| --- | --- |
| `translate [TEXT]` | 单个请求；省略文本时读 stdin |
| `roundtrip [TEXT]` | 经多跳机器翻译翻出再翻回（`--chain`、`--preset`、`--depth`、`--pivot`） |
| `batch` | NDJSON 进、NDJSON 出，保持顺序 |
| `providers` | 各 Provider 的能力和配置状态 |
| `languages` | 规范语言码，或某个 Provider 声明的集合 |
| `keys [PROVIDER]` | 冷却与失败状态；`--reset` 清除 |
| `health` | 探测所有已配置的 Provider |
| `schema` | 所有线上类型的 JSON Schema（draft 2020-12） |
| `manifest` | CLI 操作的机器可读清单 |
| `init` | 写出配置模板 |
| `completions SHELL` | Bash/zsh/fish/PowerShell 补全 |

全局参数：`--format json|text`、`--quiet`、`--verbose`、`--race N`、`--config`、`--credentials`、`--log`。

退出码：`0` 成功，`1` 请求错误，`2` 配置错误，`3` 回退链耗尽，`4` 网络错误。

## 错误契约

| `code` | 可重试 | Agent 应采取 |
| --- | --- | --- |
| `RATE_LIMITED` | 是 | 等待 `retryAfterSecs` 后重试 |
| `QUOTA_EXHAUSTED` | 否 | 换 Key 或等配额窗口恢复 |
| `NETWORK_ERROR` | 是 | 指数退避重试 |
| `UPSTREAM_ERROR` | 仅 5xx | 5xx 重试，4xx 修输入 |
| `TIMEOUT` | 是 | 提高 `timeoutMs` 后重试，或换 Provider |
| `AUTH_FAILED` | 否 | 刷新凭据 |
| `PROVIDER_NOT_CONFIGURED` | 否 | 补凭据 |
| `PROVIDER_NOT_FOUND` | 否 | 修正 Provider 名 |
| `AUTO_DETECT_UNSUPPORTED` | 否 | 显式传 `sourceLang` |
| `LANGUAGE_PAIR_NOT_SUPPORTED` | 否 | 换 Provider 或目标语言 |
| `INVALID_INPUT` | 否 | 修正请求 |
| `CHAIN_EXHAUSTED` | 否 | 用 `--verbose` 看逐跳记录 |

完整说明：[docs/zh-cn/api/OVERVIEW.md](./docs/zh-cn/api/OVERVIEW.md)。

## 文档

| 语言 | 入口 |
| --- | --- |
| English | [docs/en/OVERVIEW.md](./docs/en/OVERVIEW.md) |
| 简体中文 | [docs/zh-cn/OVERVIEW.md](./docs/zh-cn/OVERVIEW.md) |
| 往返翻译：概览、链与预设、实测结果 | [docs/zh-cn/roundtrip/OVERVIEW.md](./docs/zh-cn/roundtrip/OVERVIEW.md) |
| Harness 接入 | [integrations/README.md](./integrations/README.md) |
| Agent 集成 | [AGENTS.md](./AGENTS.md) |

## 开发

```bash
cargo test                # 264 个测试
cargo build --release     # 开 LTO 约 5 分钟
cargo clippy --all-targets
```

新增 Provider：在 `src/providers/<name>.rs` 实现 `Provider`，网页接口把 `priority` 放在 1-12、官方 API 放在 20-27，在 `src/cli.rs::build_registry` 里注册，并把名字加进 `KNOWN_PROVIDERS`。

## 相关项目

想让 AI 写的英文读起来像人写的，见 https://github.com/w21180239/not-a-bot。

## 许可

MIT OR Apache-2.0

任选 [Apache License 2.0](LICENSE-APACHE) 或 [MIT 许可](LICENSE-MIT) 其一使用。
