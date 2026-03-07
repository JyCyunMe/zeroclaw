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
| Guild 消息 | 是 | ✅ 需要 @bot |
| Guild 消息 | 否 | ✅ 直接响应 |

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
    typing_handles: Mutex<HashMap<String, tokio::task::JoinHandle<()>>>,
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

## 3. 关键代码位置参考

### Discord Channel 核心文件
```
src/channels/discord.rs
├── DiscordChannel struct (line 12-23)
├── normalize_incoming_content() (line 427-456)
├── listen() 事件处理 (line 620-985)
├── pairing 逻辑 (line 776-855)
└── 测试 (line 1060+)
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
└── ChannelCommands enum (line 94-151)
```

### 命令处理
```
src/channels/mod.rs
├── bind_telegram_identity() (line 2496-2560)
├── bind_discord_identity() (line 2562-2610)
└── 命令分发 (line 2749-2756)
```

---

## 4. 待改进项 (TODO)

### Slash Command 支持
Discord channel 当前不支持 slash command。要实现需要：
1. 扩展 Gateway Intents（添加 `GUILD_APPLICATION_COMMANDS`）
2. 处理 `INTERACTION_CREATE` 事件
3. 实现 Interaction 回调响应
4. Command 注册机制

### 测试覆盖
- 需要 integration test 验证 pairing 流程
- 需要 mock Discord Gateway 进行测试
