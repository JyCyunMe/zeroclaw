# Development Notes

## Date: 2026-03-08

---

## 1. Discord Slash Commands Phase 1 实施细节

### Problem Description
- **需求**: 扩展 Discord slash commands，增加 P1.1 和 P1.2 的 6 个新命令
- **现有基础**: 4 个基础命令（`/models`, `/model`, `/new`, `/bind`）
- **目标**: 提升用户体验，暴露更多 ZeroClaw 功能

### Implementation Summary

**新增命令**:
1. `/help` - 显示帮助信息和使用指南
2. `/commands` - 列出所有可用的 slash 命令
3. `/whoami` - 显示用户的 Discord ID 和用户名
4. `/skills` - 列出所有已安装的 skills
5. `/skill <name> [input]` - 请求执行指定的 skill
6. `/verbose on/off` - 切换详细模式

**关键技术点**:
- DiscordInteraction 扩展用户信息支持（member/user 字段）
- Skills 集成（从 workspace 加载）
- 命令分类（慢命令 vs 快命令）

### Files Modified
- `src/channels/discord_slash.rs` - 命令定义 + 用户信息结构体
- `src/channels/discord.rs` - 命令处理逻辑 + get_skills_list()

### Commit
- `58f7e0bd` feat(discord): add /help, /commands, /whoami, /skills, /skill, /verbose slash commands

### Next Steps
参考 `docs/vibe/zeroclaw-slash-commands-expansion-plan.md`:
- Phase 1.3: `/status`, `/debug` (需要 AgentStateSnapshot)
- Phase 1.4: `/config` (需要 OwnerConfig)
- Phase 2: Session 管理（`/reset`, `/new`, `/stop`）

---
