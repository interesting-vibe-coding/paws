[English](README.md) | 中文

# 🐾 Paws

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE) [![Built for Kaku](https://img.shields.io/badge/Built_for-Kaku-blue)](https://github.com/tw93/kaku) [![Made with Lua & Rust](https://img.shields.io/badge/Made_with-Lua_&_Rust-orange)]() [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/MisterBrookT/paws/pulls) [![GitHub stars](https://img.shields.io/github/stars/MisterBrookT/paws?style=social)](https://github.com/MisterBrookT/paws)

> Agent 需要你时自动切回，Agent 工作时尽情玩耍。

AI 编程 Agent 的终端伴侣。Paws 在 Agent 工作时给你一个沉浸式全屏游戏——Agent 需要你输入的瞬间自动切回。

专为 vibe coding 中被忽视的那段时间而生：你想守在终端旁，但 Agent 正在思考，你无事可做。

## 工作原理

```
       按下 CMD+G                      Agent 完成一轮
  ┌──────────────────┐            ┌──────────────────────────┐
  │  🎮 游戏标签页     │  ───────>  │  🤖 Agent 标签页          │
  │  (全窗口)         │  <───────  │  (你的分屏布局)           │
  └──────────────────┘   CMD+G     └──────────────────────────┘
```

### 快捷键

| 按键 | 功能 |
|------|------|
| **CMD+G** | 首次按下：选择游戏（🎲 随机 / 2048 / 数独 / 俄罗斯方块）；之后：在 Agent ↔ 游戏间切换 |
| **CMD+SHIFT+P** | 重新打开菜单，换一个游戏 |

> 故意不用 `CMD+SHIFT+G`——Kaku 已把它绑给了 lazygit。

### 模式

- **手动** — 按 CMD+G 跳到游戏。任意 Agent session 完成时（`stop`），Paws 把你切回那个 session 的标签页。
- **自动** — 游戏一旦打开，Agent 开始工作时（`userPromptSubmit`）自动跳到游戏，完成时自动切回。无需动手。

游戏运行在独立的**标签页**中，天然全窗口沉浸——你现有的分屏布局不会被打扰。

## 设计哲学

一切运行在终端自身的原生扩展层内。**没有外部控制脚本，没有临时文件，不调用 `kaku cli`。**

```
Kiro hooks ─ 一行 OSC 1337 user-var 发射器（stop + userPromptSubmit）
       │
       ▼
Kaku Lua ─ 大脑。通过 user-var-changed 响应，用 wezterm.mux 切换标签页，
       │   状态存在 wezterm.GLOBAL 中——全部 in-process
       ▼
游戏标签页 ─ 运行 `paws`，一个小巧的 Rust 启动器，在你已安装的游戏间
       │   轮换（每天换一款）
```

标签页归终端管，所以标签页控制就该在终端的 Lua 层——而不是从外部伸手进来。

## 环境要求

- [Kaku 终端](https://github.com/tw93/kaku)（WezTerm 分支）
- [Kiro CLI](https://kiro.dev)（主要支持）或 Claude Code（计划中）
- Rust 工具链（`cargo`），用于构建 `paws` 启动器
- 一款或多款终端游戏 — 如 `brew install c2048 nudoku vitetris`

## 安装

### 简单方式 — 让你的 Agent 来装

Paws 自带安装 skill。克隆仓库后直接告诉你的 AI 编程 Agent：

> "用 `paws/skills/paws-install/SKILL.md` 里的 skill 安装 Paws。"

Agent 会把 Lua 合并到你的 Kaku 配置、接好 hooks、装好游戏，然后提示你重载。无需手动编辑。（Kiro 原生读取 `SKILL.md`；Claude Code 也能读。）

### 手动方式

1. 构建启动器：`cargo install --path .`（在 PATH 中生成 `paws`）。
2. 将 [`lua/paws.lua`](lua/paws.lua) 添加到 `~/.config/kaku/kaku.lua`（放在 `return config` 之前）。
3. 将 [`hooks/kiro/paws-signal.sh`](hooks/kiro/paws-signal.sh) 配置为 Kiro 的 `stop` 和 `userPromptSubmit` hooks（使用**绝对路径**，注意 `done`/`busy` 参数）：
   ```json
   "hooks": {
     "stop":             [{ "command": "/absolute/path/to/paws-signal.sh done" }],
     "userPromptSubmit": [{ "command": "/absolute/path/to/paws-signal.sh busy" }]
   }
   ```
4. `brew install c2048 nudoku vitetris`，重载 Kaku（CMD+Shift+R），按 **CMD+G** 即可。

## 路线图

### 已完成
- [x] 原生标签页切换（纯 Lua，`wezterm.mux`，`wezterm.GLOBAL`）
- [x] 游戏选择菜单 `InputSelector`（CMD+G 首次；CMD+SHIFT+P 重选）
- [x] 通过 Agent skill 一键安装
- [x] `paws` Rust 启动器，每日轮换游戏（2048 / 数独 / 俄罗斯方块）

### 接下来（按优先级）
1. **在设备上验证自动切回** — 确认 Kiro hook 发出的 OSC `user-var` 信号能被 Kaku 收到（`userPromptSubmit` → 游戏，`stop` → Agent）。
2. **暂停覆盖层** — 暂停游戏并显示覆盖层 + 自动返回倒计时。
3. **更多更好的游戏** — 扩充精选列表；更丰富的游戏到位后替换掉 2048。
4. **Claude Code 支持** — notification / stop hooks。

## 设计文档

完整设计思路见 [`docs/design.tex`](docs/design.tex)。

## 许可证

MIT
