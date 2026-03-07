# Development Notes

## Date: 2026-03-07

---

## 1. Discord mention_only 对 DM 私聊的影响

### Problem Description
- **File**: `src/channels/discord.rs`
- **Symptom**: 当 `mention_only = true` 时，Discord 私聊（DM）也需要 `@bot` 才能触发响应，不符合常规 Discord bot 行为
- **Root Cause**: `normalize_incoming_content` 函数的 `mention_only` 检查不区分 DM 和 Guild 消息，对所有消息类型一视同仁

### Solution
修改 `normalize_incoming_content` 函数，新增 `is_dm` 参数：
- DM 消息（`guild_id` 为 `None`）不受 `mention_only` 限制
- Guild 消息保持原有 `mention_only` 行为

```rust
fn normalize_incoming_content(
    content: &str,
    mention_only: bool,
    bot_user_id: &str,
    is_dm: bool,  // 新增参数
) -> Option<String> {
    // mention_only 只对 Guild 消息生效，不对 DM 生效
    let require_mention = mention_only && !is_dm;
    
    if require_mention && !contains_bot_mention(content, bot_user_id) {
        return None;
    }
    // ...
}
```

调用处判断 `is_dm`:
```rust
let is_dm = d.get("guild_id").is_none();
let Some(clean_content) =
    normalize_incoming_content(content, self.mention_only, &bot_user_id, is_dm)
else {
    continue;
};
```

### Behavior After Fix
| 场景 | mention_only=true | 结果 |
|------|-------------------|------|
| DM 消息 | 是 | ✅ 直接响应（无需 @bot） |
| DM 消息 | 否 | ✅ 直接响应 |

---

## 3. Discord Slash Commands 支持

### Problem Description
- **需求**: Discord 用户期望使用原生 slash commands（如 `/models`, `/model`, `/new`, `/bind`）而非文本命令
- **当前状态**: Discord 只有文本命令匹配，无 slash command 注册和交互处理

### Implementation

采用**模块化方案**，将 slash commands 逻辑独立到 `src/channels/discord_slash.rs`:

#### 1. 核心数据结构
```rust
// Discord Application Command 注册结构
pub struct DiscordCommand {
    pub name: String,
    pub description: String,
    pub options: Option<Vec<DiscordCommandOption>>,
    pub type_: i32,  // CHAT_INPUT = 1
}

// Discord Interaction 接收结构
pub struct DiscordInteraction {
    pub id: String,
    pub interaction_type: i32,  // APPLICATION_COMMAND = 2
    pub data: Option<InteractionData>,
    pub channel_id: String,
    pub token: String,
}
```

#### 2. 命令注册（启动时）
```rust
async fn register_slash_commands(&self) -> anyhow::Result<()> {
    let commands = zeroclaw_slash_commands();  // 返回 Vec<DiscordCommand>
    let url = format!("https://discord.com/api/v10/applications/{}/commands", app_id);
    
    client.put(&url)
        .header("Authorization", format!("Bot {}", token))
        .json(&commands)
        .send()
        .await?;
}
```

#### 3. Gateway 事件处理
```rust
// 在 listen() 方法的 tokio::select! 循环中
match event_type {
    "MESSAGE_CREATE" => { /* 现有文本消息处理 */ }
    "INTERACTION_CREATE" => {
        self.handle_interaction(d).await?;
    }
    _ => {}
}
```

#### 4. 命令执行与响应
```rust
async fn handle_interaction(&self, data: &Value) -> anyhow::Result<()> {
    let interaction: DiscordInteraction = serde_json::from_value(data)?;
    
    let response = match cmd_data.name.as_str() {
        "models" => /* 调用 /models 逻辑 */,
        "model" => "当前模型信息",
        "new" => "新会话已启动",
        "bind" => format!("绑定码: {code}"),
        _ => "未知命令",
    };
    
    // 发送交互回调（3秒内必须响应）
    let callback = InteractionCallback {
        callback_type: CHANNEL_MESSAGE_WITH_SOURCE,  // = 4
        data: Some(CallbackData { content: response }),
    };
    
    client.post(&url).json(&callback).send().await?;
}
```

#### 5. 配置选项
```toml
[channels_config.discord]
enable_slash_commands = true  # 默认启用，可禁用
```

### Architecture Benefits
1. **模块隔离**: `discord_slash.rs` 独立模块，降低主文件复杂度
2. **YAGNI 原则**: 仅 Discord 需要，未抽象到 Channel trait（仅 1 caller）
3. **类型安全**: Rust struct 确保 Discord API 类型正确性
4. **可测试性**: 独立模块可单独测试

### Discord Slash Commands 完整列表
| 命令 | 描述 | 参数 |
|------|------|------|
| `/models` | 列出可用 AI 模型 | 无 |
| `/model` | 显示当前 AI 模型 | 无 |
| `/new` | 开始新会话 | 无 |
| `/bind <code>` | 使用配对码绑定账户 | `code`: 6位配对码 |

### Gateway Intents
现有 intents (37377) 已足够，无需添加：
- `GUILDS` (1): slash commands 全局注册
- `GUILD_MESSAGES` (1<<4): MESSAGE_CREATE 事件
- `MESSAGE_CONTENT` (1<<15): 文本命令
- `DIRECT_MESSAGES` (1<<12): DM 支持

### Known Limitations
1. **/models 命令**: 当前创建独立 ChannelMessage 发送到 agent，可能需要与主消息队列集成
2. **响应时间**: Discord 要求 3 秒内响应交互，复杂命令可能需要 deferred response (type 5)
3. **权限范围**: Slash commands 在 Guild 全局可见，需要管理员权限注册

### Files Modified
- `src/channels/discord_slash.rs` (新建): slash command 数据结构和工具函数
- `src/channels/discord.rs`: 
  - 添加 `enable_slash_commands` 字段
  - 实现 `register_slash_commands()` 方法
  - 实现 `handle_interaction()` 方法
  - 修改 `listen()` 处理 INTERACTION_CREATE 事件
- `src/config/schema.rs`: DiscordConfig 添加 `enable_slash_commands` 字段
- `src/channels/mod.rs`: 通道工厂传递 `enable_slash_commands` 参数
- `src/onboard/wizard.rs`: 配置向导设置默认值
- `src/cron/scheduler.rs`: 测试代码添加第6参数

### Testing
```bash
# 单元测试（模块内部）
cargo test --lib discord_slash

# 集成测试（需要真实 Discord bot）
# 1. 配置 config.toml 中 [channels_config.discord]
# 2. 运行 zeroclaw daemon
# 3. Discord 中输入 /models 查看响应
```

---


---

## 2. Discord Pairing 机制实现

### Problem Description
- **File**: `src/channels/discord.rs`
- **Symptom**: Discord channel 缺少 pairing 机制，无法支持运行时用户绑定
- **Root Cause**: 之前只依赖静态配置的 `allowed_users`，没有动态绑定能力

### Solution
参考 Telegram 的 Pairing 实现模式，为 Discord 添加完整的 pairing 支持：

#### 结构变化
```rust
pub struct DiscordChannel {
    bot_token: String,
    guild_id: Option<String>,
    allowed_users: Arc<RwLock<Vec<String>>>,  // 改为线程安全可修改
    listen_to_bots: bool,
    mention_only: bool,
    pairing: Option<PairingGuard>,  // 新增
}
```

#### 新增方法
- `is_user_allowed()` - 异步检查用户是否在 allowlist
- `add_allowed_user()` - 运行时添加用户到 allowlist
- `persist_allowed_user()` - 持久化用户到 config.toml
- `pairing_code_active()` - 检查 pairing 是否激活
- `extract_bind_code()` - 从消息中提取 `/bind <code>`
- `load_config_without_env()` - 加载配置（不依赖环境变量）

#### 绑定流程
1. `allowed_users` 为空时，启动 pairing 模式
2. 打印 6 位配对码到终端
3. 用户发送 `/bind <code>` 完成绑定
4. 自动添加用户 ID 到 allowlist 并持久化

#### CLI 命令
```bash
zeroclaw channel bind-discord <user_id>
```

### Files Changed
- `src/channels/discord.rs` - 核心实现
- `src/lib.rs` - CLI 命令定义 (`BindDiscord`)
- `src/channels/mod.rs` - 命令处理逻辑 (`bind_discord_identity`)

---

## 3. Discord Slash Command 支持

### Problem Description
- **File**: `src/channels/discord.rs`
- **Symptom**: Discord 不支持原生 Slash Command，只有文本匹配命令
- **Root Cause**: 未实现 `INTERACTION_CREATE` 事件处理和 Application Command 注册

### Solution
实现完整的 Discord Slash Command 支持：

#### 数据结构（新增模块）
```rust
mod slash_command {
    pub const CHAT_INPUT: i32 = 1;
    
    pub mod option_type {
        pub const STRING: i32 = 3;
        pub const INTEGER: i32 = 4;
        pub const BOOLEAN: i32 = 5;
        // ...
    }
    
    pub mod interaction_type {
        pub const APPLICATION_COMMAND: i32 = 2;
    }
    
    pub mod callback_type {
        pub const CHANNEL_MESSAGE_WITH_SOURCE: i32 = 4;
    }
}

struct DiscordCommand {
    name: String,
    description: String,
    command_type: Option<i32>,
    options: Vec<DiscordCommandOption>,
}

struct DiscordInteraction {
    id: String,
    token: String,
    interaction_type: i32,
    data: Option<DiscordInteractionData>,
    // ...
}
```

#### 命令映射
| Slash Command | Options | 功能 |
|---------------|---------|------|
| `/models` | 无 | 显示可用 providers |
| `/model` | `id: string?` | 显示/设置 model |
| `/new` | 无 | 清除对话历史 |
| `/bind` | `code: string` | 配对绑定 |

#### API 集成
- 注册命令: `PUT /applications/{app_id}/commands`
- 响应交互: `POST /interactions/{id}/{token}/callback`
- 事件处理: 在 `listen()` 中处理 `INTERACTION_CREATE`

#### 配置控制
```toml
[channels_config.discord]
enable_slash_commands = true  # 默认启用
```

### Files Changed
- `src/channels/discord.rs` - Slash Command 实现
- `src/config/schema.rs` - 添加 `enable_slash_commands` 配置

---

## 4. 关键代码位置参考

### Discord Channel 核心文件
```
src/channels/discord.rs
├── Slash Command 常量 (line 23-87)
├── DiscordCommand / DiscordInteraction 结构 (line 89-158)
├── DiscordChannel struct (line 163-170)
├── Slash Command 方法 (line 283-383)
├── listen() INTERACTION_CREATE 处理 (line 1060-1261)
└── 测试 (line 1410+)
```

### Channel Trait 定义
```
src/channels/traits.rs
├── ChannelMessage struct
├── SendMessage struct  
└── Channel trait
```

### CLI 命令定义
```
src/lib.rs
└── ChannelCommands enum (line 94-156)
```

### 命令处理
```
src/channels/mod.rs
├── bind_telegram_identity() (line 2496-2560)
├── bind_discord_identity() (line 2562-2610)
└── 命令分发 (line 2770-2781)
```

---

## 5. 待改进项 (TODO)

### Telegram Slash Command 支持
Telegram 的 `/bind` 目前是文本匹配，可以考虑：
1. 调用 `setMyCommands` API 注册命令
2. 解析 `message.entities` 识别命令
3. 添加自动补全支持

### Slash Command 功能扩展
当前实现的 4 个命令是基础映射，可以扩展：
- 参数验证和类型检查
- 命令权限控制
- 自动补全支持
- 多语言命令描述
