# ZeroClaw Development Notes

## 2026-03-08

### Discord Slash Commands Phase 1 扩展完成

**文件**: `src/channels/discord_slash.rs`, `src/channels/discord.rs`  
**提交**: `58f7e0bd`  
**状态**: ✅ 完成 (6/44 命令)

#### 实现的命令

**P1.1 纯静态命令:**
- `/help` - 显示帮助信息和使用指南（包含用户提及）
- `/commands` - 列出所有可用的 slash 命令（10 个）
- `/whoami` - 显示用户的 Discord ID 和用户名

**P1.2 Agent 转发命令:**
- `/skills` - 列出所有已安装的 skills（从 workspace 加载）
- `/skill <name> [input]` - 请求执行指定的 skill（引导用户使用消息）
- `/verbose on/off` - 切换详细模式（基础实现，需后续集成到 Agent）

#### 关键技术实现

**1. 用户身份扩展**
```rust
// DiscordInteraction 添加用户信息支持
pub struct DiscordInteraction {
    pub id: String,
    pub application_id: String,
    pub interaction_type: i32,
    pub data: Option<InteractionData>,
    pub channel_id: String,
    pub token: String,
    #[serde(default)]
    pub member: Option<DiscordMember>,  // Guild 用户
    #[serde(default)]
    pub user: Option<DiscordUser>,      // DM 用户
}

impl DiscordInteraction {
    pub fn user_id(&self) -> Option<&str> {
        self.member
            .as_ref()
            .and_then(|m| m.user.as_ref())
            .map(|u| u.id.as_str())
            .or_else(|| self.user.as_ref().map(|u| u.id.as_str()))
    }

    pub fn username(&self) -> Option<&str> {
        self.member
            .as_ref()
            .and_then(|m| m.user.as_ref())
            .and_then(|u| u.username.as_deref())
            .or_else(|| self.user.as_ref().and_then(|u| u.username.as_deref()))
    }
}
```

**2. Skills 列表加载**
```rust
async fn get_skills_list() -> String {
    let config = Self::load_config_without_env().await?;
    let skills = crate::skills::load_skills_with_config(&config.workspace_dir, &config);

    if skills.is_empty() {
        return "**No skills installed**\nAdd skills to ~/.zeroclaw/workspace/skills/".to_string();
    }

    let mut lines = vec![format!("**Available Skills ({})**", skills.len())];
    for skill in &skills {
        let desc = if skill.description.len() > 60 {
            format!("{}...", &skill.description[..57])
        } else {
            skill.description.clone()
        };
        lines.push(format!("• `{}` - {}", skill.name, desc));
    }

    lines.join("\n")
}
```

**3. 命令分类处理**
```rust
// 区分慢命令（需要延迟响应）和快命令
let is_slow_command = cmd_name == "models" || cmd_name == "skills";

if is_slow_command {
    // 发送 deferred ack (type 5) + webhook followup
    self.send_deferred_ack(&interaction.id, &interaction.token).await?;
    tokio::spawn(async move {
        let content = match cmd_name.as_str() {
            "models" => Self::get_cached_models_list().await,
            "skills" => Self::get_skills_list().await,
            _ => "Unknown".to_string(),
        };
        // webhook followup...
    });
} else {
    // 立即响应 (type 4)
    let response = match cmd_name {
        "help" => format!("**ZeroClaw Help**\nHello {user_mention}!..."),
        "commands" => "**Available Slash Commands**\n• /help...",
        // ...
    };
}
```

#### 命令总览

| 命令 | 类型 | 参数 | 响应类型 |
|------|------|------|----------|
| `/help` | 静态 | 无 | 快速 |
| `/commands` | 静态 | 无 | 快速 |
| `/whoami` | 静态 | 无 | 快速 |
| `/skills` | 动态 | 无 | 慢（Deferred） |
| `/skill` | 前置 | `name` (required), `input` (optional) | 快速 |
| `/verbose` | 开关 | `mode` (on/off) | 快速 |
| `/models` | 动态 | 无 | 慢（Deferred） |
| `/model` | 静态 | 无 | 快速 |
| `/new` | 静态 | 无 | 快速 |
| `/bind` | 绑定 | `code` (required) | 快速 |

**总计**: 10 个命令

#### 架构影响
- ✅ 模块化：slash commands 逻辑独立在 `discord_slash.rs`
- ✅ YAGNI：未抽象到 Channel trait（仅 Discord 需要）
- ✅ 类型安全：Rust struct 确保与 Discord API 兼容
- ✅ 可测试性：独立模块可单独测试

#### 待实现（参考 `docs/vibe/zeroclaw-slash-commands-expansion-plan.md`）

**Phase 1.3 (需状态共享)**:
- `/status` - 显示当前状态（需要 AgentStateSnapshot）
- `/debug show/set/unset/reset` - 调试命令

**Phase 1.4 (配置管理)**:
- `/config show/get/set/unset` - 配置管理（需要 OwnerConfig）

**Phase 2 (核心功能)**:
- `/reset`, `/new` 真实重置（需要 Session 框架）
- `/stop` - 停止当前运行
- `/model` 动态切换（需要 RuntimeConfig）
- `/temperature` - 温度参数
- `/allowlist add/remove` - 白名单管理
- `/elevated on/off/ask/full` - 提权模式

**Phase 3 (高级功能)**:
- `/compact` - 会话压缩（需要 LLM summarization）
- `/session idle/max-age` - 会话超时管理
- `/think <level>` - 思考级别
- `/reasoning on/off/stream` - 推理可见性
- `/queue <mode>` - 消息队列系统
- `/approve` - 审批系统

**进度**: 6/44 命令完成 (13.6%)

---

## 2026-03-07

### Cron 任务 delivery 自动注入

**问题：** LLM 创建 cron 任务时忘记设置 delivery 参数，导致任务执行成功但用户看不到结果（特别是执行失败时的错误信息）。

**根因：** 依赖 LLM 记住设置 delivery，不可靠。

**解决：** 在 agent tool-call loop 中自动注入 delivery 参数。

**实现位置：** `src/agent/loop_.rs` → `execute_one_tool()`

**关键代码：**
```rust
// 当调用 cron_add 且没有 delivery 时，自动注入
if call_name == "cron_add" && !has_delivery {
    if let Some((channel, target)) = channel_delivery {
        args["delivery"] = json!({
            "mode": "announce",
            "channel": channel,
            "to": target
        });
    }
}
```

**数据流：**
```
ChannelMessage (channel, reply_target)
    ↓
run_tool_call_loop(channel_delivery=Some((channel, reply_target)))
    ↓
execute_one_tool(call_name="cron_add", channel_delivery)
    ↓
自动注入 delivery 参数
```

**注意事项：**
- 只在 `cron_add` 工具上自动注入
- 只在 `delivery.mode != "announce"` 时注入
- 只在 `channel_delivery` 存在时注入（即有 channel context）

---

### Schedule::In 相对时间延迟

**问题：** LLM 需要计算精确时间戳（如 `2026-03-07T17:00:00Z`），容易出错。

**解决：** 添加 `Schedule::In` 类型，支持自然语言延迟。

**示例：**
```json
{"kind":"in","in":"5 minutes"}  // LLM 无需计算时间戳
{"kind":"in","in":"1 month"}    // 日历计算：Jan 31 + 1 month = Feb 28
```

**依赖库：** `interim` (支持日历单位：week, month, year)

**支持单位：** second(s), minute(s), hour(s), day(s), week(s), month(s), year(s)
