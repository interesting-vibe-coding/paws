[English](README.md) | 中文

<div align="center">

# 🐾 Paws

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE) [![Built for Kaku](https://img.shields.io/badge/Built_for-Kaku-blue)](https://github.com/tw93/kaku) [![Made with Lua & Rust](https://img.shields.io/badge/Made_with-Lua_&_Rust-orange)]() [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/MisterBrookT/paws/pulls) [![GitHub Stars](https://img.shields.io/github/stars/MisterBrookT/paws?style=flat&color=yellow)](https://github.com/MisterBrookT/paws/stargazers)

Agent 工作时尽情玩，需要你时一眼就看到。

</div>

<p align="center"><img src="docs/demo.gif" width="600" alt="Paws demo"></p>

AI 编程 Agent 的终端伴侣。按 CMD+G 选一个游戏，在全窗口标签页里玩。游戏内有实时状态栏，显示哪些 session 在跑、哪些已完成，完成时会闪烁提醒——你想切回去时再切。

## 使用

| 按键 | 功能 |
|------|------|
| **CMD+G** | 首次：选择游戏；之后：在 Agent ↔ 游戏间切换 |
| **CMD+SHIFT+P** | 重新打开菜单，换游戏 |

HUD 显示 session 状态（运行中 / 已完成），完成时闪烁。不会自动切换。

## 安装

**让你的 Agent 来装：**

> "用 `paws/skills/paws-install/SKILL.md` 里的 skill 安装 Paws。"

**手动方式：**

1. `cargo install --path .`
2. 将 [`lua/paws.lua`](lua/paws.lua) 添加到 `~/.config/kaku/kaku.lua`（`return config` 之前）。
3. 将 [`hooks/kiro/paws-signal.sh`](hooks/kiro/paws-signal.sh) 配置为 `stop` 和 `userPromptSubmit` hooks：
   ```json
   "hooks": {
     "stop":             [{ "command": "/absolute/path/to/paws-signal.sh done" }],
     "userPromptSubmit": [{ "command": "/absolute/path/to/paws-signal.sh busy" }]
   }
   ```
4. 装一个游戏（`brew install vitetris` 或 `cargo install --git https://github.com/MisterBrookT/paws-games`），重载 Kaku（CMD+Shift+R），按 CMD+G。

## 游戏

Tetris · [Dog Jump](https://github.com/MisterBrookT/paws-games) · Pinball（弹球）· 🌍 地球Online（现实世界支线任务）· 诗（Poetry）· 🎲 随机轮换

## 工作原理

Hook 把 session 状态写入 `/tmp/paws-sessions/` → Kaku Lua 处理 CMD+G（通过 `wezterm.mux` 创建/切换标签页）→ `paws` wrapper 把游戏居中托管在 PTY 里并渲染实时 HUD。

一切运行在终端自身的 Lua 层。没有外部脚本，没有自动切换。主动权在你。

---

更多项目 → [doabit.dev](https://doabit.dev) · 许可证：MIT
