# ZeroClaw Slash Commands 扩展计划

**文档日期**: 2026-03-07
**更新日期**: 2026-03-08
**分支**: `feat/slash-commands-discord-full`
**当前状态**: 10 个命令 (6 个新增命令已实现)

**实现进度**: Phase 1.1 ✅ | Phase 1.2 ✅ | Phase 1.3 ⏳ | Phase 1.4 ⏳

---

## 目录

1. [总览](#总览)
2. [功能模块分析](#功能模块分析)
   - [状态查询](#1-状态查询)
   - [会话管理](#2-会话管理)
   - [运行参数](#3-运行参数)
   - [配置管理](#4-配置管理)
   - [Skill 系统](#5-skill-系统)
3. [实施路线图](#实施路线图)
4. [架构决策点](#架构决策点)
5. [风险评估](#风险评估)

---

## 总览

### ZeroClaw vs OpenClaw 对比

| 项目 | 语言 | 命令数量 | 原生支持 |
|------|------|---------|---------|
| **ZeroClaw** | Rust | 4 | Discord |
| **OpenClaw** | TypeScript | 40+ | Discord, Telegram, Slack |

### 功能覆盖对比

| 功能领域 | ZeroClaw | OpenClaw | 说明 |
|----------|:--------:|:--------:|------|
| 模型切换 | ✅ | ✅ | 基础功能 |
| 会话重置 | ✅ | ✅ | `/new` |
| 用户绑定 | ✅ | ✅ | pairing 机制 |
| 子代理管理 | ❌ | ✅ | OpenClaw 特有 |
| ACP 集成 | ❌ | ✅ | OpenClaw 特有 |
| TTS/语音 | ❌ | ✅ | OpenClaw 特有 |
| Skill 系统 | ✅ (未暴露) | ✅ | ZeroClaw 已实现，需暴露 |
| 配置热更新 | ❌ | ✅ | `/config`, `/debug` |
| 多渠道切换 | ❌ | ✅ | dock 命令 |
| 详细的思考模式 | ❌ | ✅ | `/think` 多级别 |
| Bash 执行 | ❌ | ✅ | 主机命令 |

### 分类对比表

| 分类 | ZeroClaw (Rust) | OpenClaw (TypeScript) |
|------|-----------------|----------------------|
| **状态查询** | `/models`, `/model` | `/help`, `/commands`, `/status`, `/whoami`, `/context`, `/export-session`, `/models`, `/model` |
| **会话管理** | `/new` | `/reset`, `/new`, `/stop`, `/compact`, `/session idle\|max-age` |
| **子代理/ACP** | ❌ 无 | `/subagents`, `/agents`, `/focus`, `/unfocus`, `/kill`, `/steer`, `/acp` |
| **运行参数** | ❌ 无 | `/think`, `/verbose`, `/reasoning`, `/elevated`, `/model`, `/queue` |
| **配置管理** | ❌ 无 | `/config`, `/debug`, `/usage`, `/allowlist`, `/approve` |
| **用户绑定** | `/bind <code>` | `/bind` (通过 pairing 机制) |
| **渠道控制** | ❌ 无 | `/dock-telegram`, `/dock-discord`, `/dock-slack`, `/activation`, `/send` |
| **媒体/Skill** | ❌ 无 | `/tts` (Discord: `/voice`), `/skill`, `/vc join\|leave\|status` (Discord only) |
| **主机命令** | ❌ 无 | `/bash`, `! <cmd>`, `!poll`, `!stop` |

---

## 功能模块分析

---

## 1. 状态查询

### 1.1 目标命令

| 命令 | 描述 | OpenClaw 对应 |
|------|------|---------------|
| `/status` | 显示当前状态（模型、provider、会话信息） | ✅ |
| `/whoami` | 显示发送者 ID | ✅ |
| `/context` | 解释上下文构建 | ✅ |
| `/help` | 显示帮助 | ✅ |
| `/commands` | 列出所有命令 | ✅ |

### 1.2 现有基础

| 方面 | 状态 | 位置 |
|------|------|------|
| 数据源 | ✅ 已有 | `Agent`: `model_name`, `provider`, `tools`, `skills` |
| History 管理 | ✅ 已有 | `Agent.history: Vec<ConversationMessage>` |
| Memory 系统 | ✅ 已有 | `Memory` trait + 实现 |

### 1.3 关键障碍

**Slash command handler 无法访问 Agent 实例**

```
discord.rs:193-236 (handle_interaction)
├── 独立作用域
├── 无 Agent 引用
└── 无法获取运行时状态
```

当前架构:
```
┌─────────────────┐     ┌─────────────────┐
│  DiscordChannel │     │     Agent       │
│  - listen()     │────▶│  - provider     │
│  - send()       │     │  - tools        │
│  - handle_      │     │  - history      │
│    interaction()│     │  - memory       │
└─────────────────┘     └─────────────────┘
        │                      │
        │    无直接访问路径     │
        └──────────────────────┘
```

### 1.4 解决方案

```rust
// 方案: 添加全局状态共享

// 1. 定义 Agent 状态快照
pub struct AgentStateSnapshot {
    pub model_name: String,
    pub provider_name: String,
    pub tools_count: usize,
    pub skills_count: usize,
    pub history_length: usize,
    pub last_activity: SystemTime,
}

// 2. 全局状态存储
static AGENT_STATE: OnceLock<RwLock<AgentStateSnapshot>> = OnceLock::new();

// 3. Agent 定期更新
impl Agent {
    async fn turn(&mut self, ...) -> Result<String> {
        // ... 现有逻辑 ...
        
        // 更新状态快照
        self.update_state_snapshot();
    }
    
    fn update_state_snapshot(&self) {
        if let Some(state) = AGENT_STATE.get() {
            let snapshot = AgentStateSnapshot {
                model_name: self.model_name.clone(),
                provider_name: self.provider.name().to_string(),
                tools_count: self.tools.len(),
                skills_count: self.skills.len(),
                history_length: self.history.len(),
                last_activity: SystemTime::now(),
            };
            let _ = state.write().map(|mut w| *w = snapshot);
        }
    }
}

// 4. Slash handler 读取
impl DiscordChannel {
    async fn handle_interaction(&self, ...) -> Result<()> {
        match cmd_name {
            "status" => {
                let state = AGENT_STATE.get()
                    .and_then(|s| s.read().ok())
                    .map(|r| format!(
                        "Model: {}\nProvider: {}\nTools: {}\nSkills: {}",
                        r.model_name, r.provider_name, r.tools_count, r.skills_count
                    ))
                    .unwrap_or_else(|| "Agent not initialized".to_string());
                // ... 响应
            }
        }
    }
}
```

### 1.5 复杂度评估

| 方面 | 评分 | 说明 |
|------|------|------|
| **整体复杂度** | ★★☆☆☆ | 中低 |
| **代码改动量** | 小 | 约 100-150 行 |
| **架构影响** | 小 | 添加状态共享，不改变现有流程 |
| **测试难度** | 低 | 可独立测试 |

### 1.6 实施步骤

```markdown
- [ ] Step 1: 定义 AgentStateSnapshot 结构
        位置: src/agent/state.rs (新建)
        工作量: 30 分钟

- [ ] Step 2: 实现全局状态存储
        位置: src/agent/mod.rs
        工作量: 30 分钟

- [ ] Step 3: Agent 集成状态更新
        位置: src/agent/agent.rs (turn 方法)
        工作量: 1 小时

- [ ] Step 4: 添加 slash command 定义
        位置: src/channels/discord_slash.rs
        工作量: 30 分钟

- [ ] Step 5: 实现 command handler
        位置: src/channels/discord.rs (handle_interaction)
        工作量: 1-2 小时

- [ ] Step 6: 测试验证
        工作量: 1 小时
```

### 1.7 预估工时

| 命令 | 工时 |
|------|------|
| `/status` | 2-3 小时 |
| `/whoami` | 30 分钟 (仅读取 interaction.user_id) |
| `/help` | 1 小时 (静态文本) |
| `/commands` | 1 小时 (遍历已注册命令) |
| `/context` | 4-6 小时 (需要深入 Agent 内部) |

**总计**: 8-12 小时

---

## 2. 会话管理

### 2.1 目标命令

| 命令 | 描述 | OpenClaw 对应 |
|------|------|---------------|
| `/reset` | 重置当前会话 | ✅ |
| `/new` | 开始新会话 | ✅ (已有) |
| `/stop` | 停止当前运行 | ✅ |
| `/compact` | 压缩会话上下文 | ✅ |
| `/session idle` | 管理会话空闲超时 | ✅ |
| `/session max-age` | 管理会话最大存活时间 | ✅ |

### 2.2 现有基础

| 方面 | 状态 | 位置 |
|------|------|------|
| History 管理 | ✅ 已有 | `Agent.history: Vec<ConversationMessage>` |
| History 裁剪 | ✅ 已有 | `Agent.trim_history()` |
| Memory 持久化 | ✅ 已有 | `Memory` trait |
| `/new` 命令 | ⚠️ 伪实现 | 仅返回文本，未真正重置 |

### 2.3 关键障碍

**ZeroClaw 无 Session 抽象**

```
当前架构:
┌─────────────────────────────────────┐
│           单一 Agent 实例            │
│  - history 在 turn() 之间不持久     │
│  - 无 session key 概念              │
│  - 无多 session 并发支持            │
└─────────────────────────────────────┘

OpenClaw 架构:
┌─────────────────────────────────────┐
│          SessionManager             │
│  - 多 session 并发管理              │
│  - session key (用户+渠道)          │
│  - 独立的 history/memory            │
│  - 空闲超时自动清理                 │
└─────────────────────────────────────┘
```

当前 `/new` 实现 (伪代码):
```rust
// discord.rs:221
"new" => "New session started".to_string(),
// 问题: 仅返回文本，未真正重置 Agent.history
```

### 2.4 解决方案

```rust
// 方案: 引入 SessionManager

// 1. Session 定义
pub struct Session {
    pub key: SessionKey,
    pub agent: Agent,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub max_age: Option<Duration>,
    pub idle_timeout: Option<Duration>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct SessionKey {
    pub channel: String,      // "discord", "telegram"
    pub user_id: String,
    pub thread_id: Option<String>,
}

// 2. SessionManager
pub struct SessionManager {
    sessions: RwLock<HashMap<SessionKey, Session>>,
    config: SessionConfig,
}

impl SessionManager {
    pub fn get_or_create(&self, key: &SessionKey) -> Arc<Mutex<Session>> {
        // 检查是否存在 + 未过期
        // 否则创建新 session
    }
    
    pub fn reset(&self, key: &SessionKey) -> Result<()> {
        // 清空 history
        // 保留 memory
    }
    
    pub fn compact(&self, key: &SessionKey) -> Result<String> {
        // LLM summarization
        // 替换 history 为 summary
    }
    
    pub fn cleanup_expired(&self) {
        // 后台任务清理过期 session
    }
}

// 3. 全局 SessionManager
static SESSION_MANAGER: OnceLock<SessionManager> = OnceLock::new();

// 4. Discord 集成
impl DiscordChannel {
    async fn handle_interaction(&self, interaction: &DiscordInteraction) -> Result<()> {
        let session_key = SessionKey {
            channel: "discord".to_string(),
            user_id: interaction.user_id(),
            thread_id: interaction.thread_id(),
        };
        
        match cmd_name {
            "new" => {
                SESSION_MANAGER.get().reset(&session_key)?;
                "New session started".to_string()
            }
            "reset" => {
                SESSION_MANAGER.get().reset(&session_key)?;
                "Session reset".to_string()
            }
            "compact" => {
                let summary = SESSION_MANAGER.get().compact(&session_key)?;
                format!("Session compacted: {}", summary)
            }
        }
    }
}
```

### 2.5 复杂度评估

| 方面 | 评分 | 说明 |
|------|------|------|
| **整体复杂度** | ★★★☆☆ | 中高 |
| **代码改动量** | 大 | 约 500-800 行新代码 |
| **架构影响** | 大 | 引入新核心组件 SessionManager |
| **测试难度** | 中 | 需要 mock Agent 和 Memory |

### 2.6 实施步骤

```markdown
Phase 1: Session 基础 (不含 /compact)
─────────────────────────────────────
- [ ] Step 1: 定义 Session 和 SessionKey
        位置: src/session/mod.rs (新建)
        工作量: 2 小时

- [ ] Step 2: 实现 SessionManager
        位置: src/session/manager.rs
        工作量: 4 小时

- [ ] Step 3: 集成到 Agent 创建流程
        位置: src/agent/agent.rs (from_config)
        工作量: 2 小时

- [ ] Step 4: 实现 /reset, /new 真实重置
        位置: src/channels/discord.rs
        工作量: 2 小时

- [ ] Step 5: 实现过期清理
        位置: src/session/cleanup.rs
        工作量: 2 小时

Phase 1 小计: 12 小时

Phase 2: /compact 实现
─────────────────────────────────────
- [ ] Step 6: 实现 LLM summarization
        位置: src/session/compaction.rs
        工作量: 8-12 小时

- [ ] Step 7: 集成到 slash command
        工作量: 2 小时

Phase 2 小计: 10-14 小时
```
Phase 1: Session 基础 (不含 /compact)
─────────────────────────────────────
Step 1: 定义 Session 和 SessionKey
        位置: src/session/mod.rs (新建)
        工作量: 2 小时

Step 2: 实现 SessionManager
        位置: src/session/manager.rs
        工作量: 4 小时

Step 3: 集成到 Agent 创建流程
        位置: src/agent/agent.rs (from_config)
        工作量: 2 小时

Step 4: 实现 /reset, /new 真实重置
        位置: src/channels/discord.rs
        工作量: 2 小时

Step 5: 实现过期清理
        位置: src/session/cleanup.rs
        工作量: 2 小时

Phase 1 小计: 12 小时

Phase 2: /compact 实现
─────────────────────────────────────
Step 6: 实现 LLM summarization
        位置: src/session/compaction.rs
        工作量: 8-12 小时

Step 7: 集成到 slash command
        工作量: 2 小时

Phase 2 小计: 10-14 小时
```

### 2.7 预估工时

| 功能 | 工时 |
|------|------|
| Session 基础框架 | 8-12 小时 |
| `/reset`, `/new` 真实实现 | 2-4 小时 |
| `/stop` | 2-4 小时 |
| `/compact` | 10-14 小时 |
| `/session idle/max-age` | 4-6 小时 |

**总计**: 26-40 小时

---

## 3. 运行参数

### 3.1 目标命令

| 命令 | 描述 | OpenClaw 对应 |
|------|------|---------------|
| `/think <level>` | 设置思考级别 | ✅ |
| `/verbose on/off` | 切换详细模式 | ✅ |
| `/reasoning on/off/stream` | 推理可见性 | ✅ |
| `/elevated on/off/ask/full` | 提权模式 | ✅ |
| `/model <name>` | 切换模型 | ✅ (已有基础) |
| `/queue <mode>` | 队列设置 | ✅ |

### 3.2 现有基础

| 方面 | 状态 | 位置 |
|------|------|------|
| Config 结构 | ✅ 完整 | `Config` + `AgentConfig` |
| Temperature | ✅ 已有 | `Agent.temperature` |
| Model 选择 | ✅ 已有 | `Agent.model_name` |
| Provider 切换 | ⚠️ 部分 | 需要重新创建 Agent |

### 3.3 关键障碍

**Config 在 Agent 启动时加载，之后不可变**

```rust
// agent.rs:320-345
Agent::builder()
    .provider(provider)      // 创建时固定
    .model_name(model_name)  // 创建时固定
    .temperature(config.default_temperature)  // 创建时固定
    .build()

// 问题: 所有参数在 build() 后不可变
```

**Provider 切换需要重建 Agent**

```rust
// 当前无法在运行时切换 provider
// 因为 Provider trait 对象已经 boxed
provider: Box<dyn Provider>  // 不可变
```

### 3.4 解决方案

```rust
// 方案 1: RuntimeConfig 覆盖层 (简单参数)

pub struct RuntimeConfig {
    overrides: RwLock<ConfigOverrides>,
}

pub struct ConfigOverrides {
    model: Option<String>,
    temperature: Option<f64>,
    verbose: bool,
    thinking_level: Option<ThinkingLevel>,
    reasoning_mode: ReasoningMode,
}

impl Agent {
    fn effective_temperature(&self) -> f64 {
        self.runtime_config.overrides.read()
            .and_then(|o| o.temperature)
            .unwrap_or(self.temperature)
    }
}

// 方案 2: Agent 动态重建 (Provider 切换)

impl SessionManager {
    pub fn switch_provider(&self, key: &SessionKey, provider_id: &str) -> Result<()> {
        let mut session = self.get_session(key)?;
        
        // 保存当前 history
        let history = session.agent.take_history();
        
        // 创建新 Agent
        let new_agent = Agent::from_config_with_provider(
            &self.config,
            provider_id,
        )?;
        
        // 恢复 history
        new_agent.set_history(history);
        
        // 替换
        session.agent = new_agent;
        Ok(())
    }
}

// 方案 3: Queue 系统 (全新子系统)

pub struct MessageQueue {
    messages: VecDeque<QueuedMessage>,
    mode: QueueMode,
    debounce: Option<Duration>,
    cap: Option<usize>,
}

pub enum QueueMode {
    Steer,       // 后续消息 steering 当前执行
    Interrupt,   // 中断当前，处理新消息
    Followup,    // 排队等待
    Collect,     // 收集后批量处理
}

impl DiscordChannel {
    async fn handle_message(&self, msg: &Message) {
        // 入队
        self.queue.enqueue(msg);
        
        // 根据 mode 决定处理方式
        match self.queue.mode {
            QueueMode::Interrupt => self.interrupt_current(),
            QueueMode::Steer => self.steer_current(msg),
            // ...
        }
    }
}
```

### 3.5 复杂度评估

| 命令 | 复杂度 | 说明 |
|------|--------|------|
| `/model` | ★★☆☆☆ | Session 重建 + history 保留 |
| `/verbose` | ★☆☆☆☆ | 仅日志级别控制 |
| `/temperature` | ★☆☆☆☆ | RuntimeConfig 覆盖 |
| `/think` | ★★★★☆ | 需要 Provider 支持 extended thinking |
| `/reasoning` | ★★★☆☆ | 需要修改响应处理流程 |
| `/elevated` | ★★☆☆☆ | SecurityPolicy 运行时修改 |
| `/queue` | ★★★★★ | 全新消息调度子系统 |

### 3.6 实施步骤

```markdown
Phase 1: 简单参数 (不含 queue/think)
─────────────────────────────────────
- [ ] Step 1: 实现 RuntimeConfig
        位置: src/config/runtime.rs (新建)
        工作量: 2 小时

- [ ] Step 2: Agent 支持运行时读取
        位置: src/agent/agent.rs
        工作量: 2 小时

- [ ] Step 3: 实现 /model 切换
        位置: src/session/manager.rs + discord.rs
        工作量: 4 小时

- [ ] Step 4: 实现 /verbose, /temperature
        位置: src/channels/discord.rs
        工作量: 2 小时

Phase 1 小计: 10 小时

Phase 2: 高级参数
─────────────────────────────────────
- [ ] Step 5: 实现 /elevated
        位置: src/security/runtime.rs
        工作量: 4 小时

- [ ] Step 6: 实现 /reasoning
        位置: src/agent/response.rs
        工作量: 6-8 小时

- [ ] Step 7: 实现 /think (extended thinking)
        位置: src/providers/extended_thinking.rs
        工作量: 16-24 小时 (依赖 Provider)

Phase 2 小计: 26-36 小时

Phase 3: Queue 系统
─────────────────────────────────────
- [ ] Step 8: 设计 Queue 架构
        工作量: 4 小时

- [ ] Step 9: 实现 MessageQueue
        位置: src/channel/queue.rs
        工作量: 12 小时

- [ ] Step 10: 集成到 Discord channel
         工作量: 8 小时

Phase 3 小计: 24-40 小时
```

### 3.7 预估工时

| 功能 | 工时 |
|------|------|
| RuntimeConfig 基础 | 4-6 小时 |
| `/model` 动态切换 | 4-6 小时 |
| `/verbose`, `/temperature` | 2-4 小时 |
| `/elevated` | 4-6 小时 |
| `/reasoning` | 6-8 小时 |
| `/think` (extended thinking) | 16-24 小时 |
| `/queue` | 24-40 小时 |

**总计**: 60-94 小时

---

## 4. 配置管理

### 4.1 目标命令

| 命令 | 描述 | OpenClaw 对应 |
|------|------|---------------|
| `/config show/get/set/unset` | 配置读写 | ✅ |
| `/debug show/set/unset/reset` | 运行时覆盖 | ✅ |
| `/allowlist add/remove` | 白名单管理 | ✅ |
| `/approve <id>` | 审批执行请求 | ✅ |

### 4.2 现有基础

| 方面 | 状态 | 位置 |
|------|------|------|
| Config 加载 | ✅ 完整 | `Config::load()` |
| Config 保存 | ✅ 完整 | `Config::save()` |
| allowed_users | ✅ 已有 | `DiscordConfig.allowed_users` |
| SecurityPolicy | ✅ 已有 | `SecurityPolicy` |

### 4.3 关键障碍

**无 Owner 身份概念**

```
当前: 所有用户权限相同
需要: 区分 owner (可修改配置) 和普通用户

OpenClaw 模式:
- commands.allowFrom 指定 owner 列表
- /config, /debug 仅 owner 可用
```

**审批队列不存在**

```
当前: 无 approval 机制
需要: 
- 执行请求队列
- pending/approved/denied 状态
- 临时权限授予
```

### 4.4 解决方案

```rust
// 方案 1: Owner 身份

// config/schema.rs
pub struct Config {
    // 新增
    pub owner: Option<OwnerConfig>,
}

pub struct OwnerConfig {
    pub discord_users: Vec<String>,  // Discord user IDs
    pub telegram_users: Vec<String>, // Telegram user IDs
}

// slash command 权限检查
impl DiscordChannel {
    fn is_owner(&self, user_id: &str) -> bool {
        self.config.owner
            .map(|o| o.discord_users.iter().any(|u| u == user_id))
            .unwrap_or(false)
    }
    
    async fn handle_interaction(&self, interaction: &DiscordInteraction) -> Result<()> {
        match cmd_name {
            "config" if !self.is_owner(&interaction.user_id()) => {
                return Ok(()); // 静默拒绝
            }
            // ...
        }
    }
}

// 方案 2: 配置热更新

impl Config {
    pub fn apply_override(&mut self, path: &str, value: Value) -> Result<()> {
        // 路径解析: "default_temperature" -> self.default_temperature
        // 验证值合法性
        // 应用修改
    }
    
    pub async fn save(&self) -> Result<()> {
        // 原子写入 config.toml
    }
}

// 方案 3: 审批队列

pub struct ApprovalQueue {
    pending: RwLock<Vec<ApprovalRequest>>,
}

pub struct ApprovalRequest {
    pub id: String,
    pub tool: String,
    pub args: Value,
    pub requester_id: String,
    pub created_at: SystemTime,
    pub status: ApprovalStatus,
}

pub enum ApprovalStatus {
    Pending,
    Approved { approver_id: String },
    Denied { approver_id: String, reason: String },
}

impl ApprovalQueue {
    pub fn create_request(&self, tool: &str, args: Value, requester: &str) -> String {
        // 创建请求，返回 ID
    }
    
    pub fn approve(&self, id: &str, approver: &str) -> Result<()> {
        // 执行工具
    }
    
    pub fn deny(&self, id: &str, approver: &str, reason: &str) -> Result<()> {
        // 拒绝并通知请求者
    }
}
```

### 4.5 复杂度评估

| 命令 | 复杂度 | 说明 |
|------|--------|------|
| `/config` | ★★☆☆☆ | 已有 save/load，需添加热更新 |
| `/debug` | ★★☆☆☆ | 运行时覆盖层 (类似 RuntimeConfig) |
| `/allowlist` | ★★☆☆☆ | 修改运行时列表 + 持久化 |
| `/approve` | ★★★★☆ | 全新审批子系统 |

### 4.6 实施步骤

```markdown
Phase 1: Owner 身份 + /config, /debug
─────────────────────────────────────
- [ ] Step 1: 添加 OwnerConfig
        位置: src/config/schema.rs
        工作量: 1 小时

- [ ] Step 2: 实现权限检查
        位置: src/channels/discord.rs
        工作量: 1 小时

- [ ] Step 3: 实现配置热更新
        位置: src/config/hot_reload.rs (新建)
        工作量: 2 小时

- [ ] Step 4: 实现 /config 命令
        位置: src/channels/discord.rs
        工作量: 2 小时

- [ ] Step 5: 实现 /debug 命令
        工作量: 2 小时

Phase 1 小计: 8 小时

Phase 2: /allowlist
─────────────────────────────────────
- [ ] Step 6: 运行时 allowlist 修改
        位置: src/channels/discord.rs
        工作量: 2 小时

- [ ] Step 7: 持久化到配置
        工作量: 2 小时

Phase 2 小计: 4 小时

Phase 3: /approve 审批系统
─────────────────────────────────────
- [ ] Step 8: 设计审批架构
        工作量: 2 小时

- [ ] Step 9: 实现 ApprovalQueue
        位置: src/approval/mod.rs (新建)
        工作量: 6 小时

- [ ] Step 10: 集成到 Tool 执行
         位置: src/agent/dispatcher.rs
         工作量: 4 小时

- [ ] Step 11: 实现 /approve 命令
         工作量: 4 小时

Phase 3 小计: 16 小时
```
Phase 1: Owner 身份 + /config, /debug
─────────────────────────────────────
Step 1: 添加 OwnerConfig
        位置: src/config/schema.rs
        工作量: 1 小时

Step 2: 实现权限检查
        位置: src/channels/discord.rs
        工作量: 1 小时

Step 3: 实现配置热更新
        位置: src/config/hot_reload.rs (新建)
        工作量: 2 小时

Step 4: 实现 /config 命令
        位置: src/channels/discord.rs
        工作量: 2 小时

Step 5: 实现 /debug 命令
        工作量: 2 小时

Phase 1 小计: 8 小时

Phase 2: /allowlist
─────────────────────────────────────
Step 6: 运行时 allowlist 修改
        位置: src/channels/discord.rs
        工作量: 2 小时

Step 7: 持久化到配置
        工作量: 2 小时

Phase 2 小计: 4 小时

Phase 3: /approve 审批系统
─────────────────────────────────────
Step 8: 设计审批架构
        工作量: 2 小时

Step 9: 实现 ApprovalQueue
        位置: src/approval/mod.rs (新建)
        工作量: 6 小时

Step 10: 集成到 Tool 执行
         位置: src/agent/dispatcher.rs
         工作量: 4 小时

Step 11: 实现 /approve 命令
         工作量: 4 小时

Phase 3 小计: 16 小时
```

### 4.7 预估工时

| 功能 | 工时 |
|------|------|
| Owner 身份 + 权限检查 | 2-3 小时 |
| `/config` | 4-6 小时 |
| `/debug` | 2-4 小时 |
| `/allowlist` | 4-6 小时 |
| `/approve` 审批系统 | 16-24 小时 |

**总计**: 28-43 小时

---

## 5. Skill 系统

### 5.1 目标命令

| 命令 | 描述 | OpenClaw 对应 |
|------|------|---------------|
| `/skill <name> [input]` | 运行指定技能 | ✅ |
| `/skills` | 列出可用技能 | OpenClaw 无此命令 |

### 5.2 现有基础

| 方面 | 状态 | 位置 |
|------|------|------|
| Skill 加载 | ✅ **完整** | `load_skills_with_config()` |
| Skill 结构 | ✅ 完整 | `Skill`, `SkillTool` |
| Skill 注册为工具 | ✅ 完整 | Agent 已集成 skills |
| OpenSkills 同步 | ✅ 完整 | 自动同步社区技能 |

**关键发现**: ZeroClaw 已有**完整的 Skill 系统**，仅需暴露为 slash command！

### 5.3 现有代码分析

```rust
// skills/mod.rs

pub struct Skill {
    pub name: String,
    pub description: String,
    pub version: String,
    pub tools: Vec<SkillTool>,
    pub prompts: Vec<String>,
}

pub struct SkillTool {
    pub name: String,
    pub description: String,
    pub kind: String,        // "shell", "http", "script"
    pub command: String,
    pub args: HashMap<String, String>,
}

// 已实现:
// - load_skills() - 从 workspace/skills/ 加载
// - load_skills_with_config() - 支持配置
// - ensure_open_skills_repo() - 自动同步社区技能
// - SkillTool 作为 Tool 执行
```

### 5.4 解决方案

```rust
// 极简实现: 添加 slash command

// 1. discord_slash.rs
pub fn zeroclaw_slash_commands() -> Vec<DiscordCommand> {
    vec![
        // 现有命令...
        
        // 新增
        DiscordCommand::new("skill", "Run a skill by name").with_options(
            vec![
                DiscordCommandOption::new("name", "Skill name", types::option_type::STRING)
                    .required(),
                DiscordCommandOption::new("input", "Skill input", types::option_type::STRING),
            ]
        ),
        DiscordCommand::new("skills", "List available skills"),
    ]
}

// 2. discord.rs
impl DiscordChannel {
    async fn handle_interaction(&self, interaction: &DiscordInteraction) -> Result<()> {
        match cmd_name {
            "skill" => {
                let skill_name = /* 从 options 提取 */;
                let input = /* 从 options 提取 */;
                
                // 方案 A: 转发到 Agent (推荐)
                let msg = ChannelMessage {
                    content: format!("/skill {} {}", skill_name, input.unwrap_or_default()),
                    // ...
                };
                self.send_to_agent(msg).await?;
                
                // 方案 B: 直接执行
                // 需要 Agent 实例访问
            }
            "skills" => {
                let skills = crate::skills::load_skills(&self.config.workspace_dir);
                skills.iter()
                    .map(|s| format!("- {}: {}", s.name, s.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}
```

### 5.5 复杂度评估

| 方面 | 评分 | 说明 |
|------|------|------|
| **整体复杂度** | ★☆☆☆☆ | 低 |
| **代码改动量** | 极小 | 约 50-80 行 |
| **架构影响** | 无 | 利用现有 Skill 系统 |
| **测试难度** | 低 | 可独立测试 |

### 5.6 实施步骤

```markdown
- [x] Step 1: 添加 slash command 定义
        位置: src/channels/discord_slash.rs
        工作量: 30 分钟

- [x] Step 2: 实现 /skills 列表命令
        位置: src/channels/discord.rs
        工作量: 30 分钟

- [x] Step 3: 实现 /skill 执行命令
        位置: src/channels/discord.rs
        工作量: 1 小时

- [x] Step 4: 测试验证
        工作量: 1 小时
```

**状态**: ✅ 已完成 (2026-03-08)

### 5.7 预估工时

| 命令 | 工时 |
|------|------|
| `/skill` | 2-3 小时 |
| `/skills` | 1 小时 |

**总计**: 3-4 小时

---

## 实施路线图

### Phase 0: 前置条件 (可选)

```markdown
- [ ] 决定是否引入 Session 抽象
  └─ 影响: Phase 2 所有功能
  └─ 建议: 引入，为未来扩展打基础
```

---

### Phase 1: 快速胜利 (低风险，高价值)

**优先级**: 🔴 最高
**总工时**: 11-16 小时
**状态**: 🔄 进行中 (40% 完成 - P1.1 + P1.2 已完成)

#### 1.1 纯静态命令 (无依赖)

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [x] | `/help` | 0.5h | 无 | `discord.rs` |
| - [x] | `/commands` | 0.5h | 无 | `discord.rs` |
| - [x] | `/whoami` | 0.25h | 无 | `discord.rs` |
| - [x] | `/skills` | 1h | 无 | `discord.rs` |

**小计**: 2.25 小时 ✅ 已完成

#### 1.2 Agent 转发命令

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [x] | `/skill <name> [input]` | 2-3h | Agent 消息队列 | `discord_slash.rs` + `discord.rs` |
| - [x] | `/verbose on/off` | 1-2h | 全局变量 | `discord.rs` |

**小计**: 3-5 小时 ✅ 已完成

#### 1.3 需状态共享

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | 添加 `AgentStateSnapshot` | 1h | 无 | `src/agent/state.rs` (新建) |
| - [ ] | 实现全局状态存储 | 0.5h | 无 | `src/agent/mod.rs` |
| - [ ] | Agent 集成状态更新 | 1h | 无 | `src/agent/agent.rs` |
| - [ ] | `/status` | 0.5h | 状态共享 | `discord.rs` |
| - [ ] | `/debug show/set/unset/reset` | 2-4h | 无 | `discord.rs` |

**小计**: 5-7 小时

#### 1.4 配置管理 (需 Owner)

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | 添加 `OwnerConfig` | 1h | 无 | `src/config/schema.rs` |
| - [ ] | 实现权限检查 | 1h | Owner | `discord.rs` |
| - [ ] | 实现配置热更新 | 2h | 无 | `src/config/hot_reload.rs` (新建) |
| - [ ] | `/config show/get/set/unset` | 2h | Owner + 热更新 | `discord.rs` |

**小计**: 6 小时

#### Phase 1 进度汇总

```
Phase 1.1 (纯静态):      [x] [x] [x] [x]  4/4  ✅ (2.25h)
Phase 1.2 (Agent 转发):  [x] [x]          2/2  ✅ (3-5h)
Phase 1.3 (状态共享):    [ ] [ ] [ ] [ ] [ ]  0/5  (5-7h)
Phase 1.4 (配置管理):    [ ] [ ] [ ] [ ]  0/4  (6h)

Phase 1 总进度:          6/15 任务        40% 完成
Phase 1 已用工时:        ~5h / 16.25-20.25h
```

---

### Phase 2: 核心功能 (中风险，中价值)

**优先级**: 🟡 中  
**总工时**: 42-62 小时  
**状态**: ⏳ 等待 Phase 1 完成

#### 2.1 Session 基础框架

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | 定义 `Session` 和 `SessionKey` | 2h | 无 | `src/session/mod.rs` (新建) |
| - [ ] | 实现 `SessionManager` | 4h | Session | `src/session/manager.rs` |
| - [ ] | 集成到 Agent 创建流程 | 2h | SessionManager | `src/agent/agent.rs` |
| - [ ] | 实现过期清理 | 2h | SessionManager | `src/session/cleanup.rs` (新建) |

**小计**: 10 小时

#### 2.2 会话管理命令

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | `/reset` 真实重置 | 1h | Session | `discord.rs` |
| - [ ] | `/new` 真实重置 | 1h | Session | `discord.rs` |
| - [ ] | `/stop` | 2-4h | Session | `discord.rs` |

**小计**: 4-6 小时

#### 2.3 运行时参数

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | 实现 `RuntimeConfig` | 2h | 无 | `src/config/runtime.rs` (新建) |
| - [ ] | Agent 支持运行时读取 | 2h | RuntimeConfig | `src/agent/agent.rs` |
| - [ ] | `/model` 动态切换 | 4h | Session | `discord.rs` |
| - [ ] | `/temperature` | 0.5h | RuntimeConfig | `discord.rs` |

**小计**: 8.5 小时

#### 2.4 安全与配置

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | `/allowlist add/remove` | 4-6h | Owner | `discord.rs` |
| - [ ] | `/elevated on/off/ask/full` | 4-6h | 无 | `discord.rs` |

**小计**: 8-12 小时

#### Phase 2 进度汇总

```
Phase 2.1 (Session 框架): [ ] [ ] [ ] [ ]  0/4  (10h)
Phase 2.2 (会话管理):     [ ] [ ] [ ]      0/3  (4-6h)
Phase 2.3 (运行时参数):   [ ] [ ] [ ] [ ]  0/4  (8.5h)
Phase 2.4 (安全配置):     [ ] [ ]          0/2  (8-12h)

Phase 2 总进度:           0/13 任务        0% 完成
Phase 2 已用工时:         0h / 30.5-36.5h
```

---

### Phase 3: 高级功能 (高风险，低价值)

**优先级**: 🟢 低  
**总工时**: 78-130 小时  
**状态**: ⏳ 等待 Phase 2 完成

#### 3.1 会话高级功能

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | LLM summarization | 8-12h | 无 | `src/session/compaction.rs` (新建) |
| - [ ] | `/compact` | 2h | summarization | `discord.rs` |
| - [ ] | `/session idle` | 2-3h | Session | `discord.rs` |
| - [ ] | `/session max-age` | 2-3h | Session | `discord.rs` |

**小计**: 14-20 小时

#### 3.2 推理与思考

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | `/reasoning on/off/stream` | 6-8h | 无 | `discord.rs` |
| - [ ] | Extended thinking 支持 | 12-16h | Provider | `src/providers/extended_thinking.rs` (新建) |
| - [ ] | `/think <level>` | 4-8h | extended thinking | `discord.rs` |

**小计**: 22-32 小时

#### 3.3 审批系统

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | 设计审批架构 | 2h | 无 | 设计文档 |
| - [ ] | 实现 `ApprovalQueue` | 6h | 无 | `src/approval/mod.rs` (新建) |
| - [ ] | 集成到 Tool 执行 | 4h | ApprovalQueue | `src/agent/dispatcher.rs` |
| - [ ] | `/approve <id>` | 4h | ApprovalQueue | `discord.rs` |

**小计**: 16 小时

#### 3.4 消息队列

| 状态 | 功能 | 工时 | 依赖 | 位置 |
|:----:|------|------|------|------|
| - [ ] | 设计 Queue 架构 | 4h | 无 | 设计文档 |
| - [ ] | 实现 `MessageQueue` | 12h | 无 | `src/channel/queue.rs` (新建) |
| - [ ] | 集成到 Discord channel | 8h | MessageQueue | `discord.rs` |
| - [ ] | `/queue <mode>` | 2-4h | MessageQueue | `discord.rs` |

**小计**: 26-28 小时

#### Phase 3 进度汇总

```
Phase 3.1 (会话高级):     [ ] [ ] [ ] [ ]  0/4  (14-20h)
Phase 3.2 (推理思考):     [ ] [ ] [ ]      0/3  (22-32h)
Phase 3.3 (审批系统):     [ ] [ ] [ ] [ ]  0/4  (16h)
Phase 3.4 (消息队列):     [ ] [ ] [ ] [ ]  0/4  (26-28h)

Phase 3 总进度:           0/15 任务        0% 完成
Phase 3 已用工时:         0h / 78-96h
```

---

## 总体进度看板

```
┌─────────────────────────────────────────────────────────────┐
│                    ZeroClaw Slash Commands                  │
│                      扩展进度总览                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Phase 1 (快速胜利)                                          │
│  ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0%   0/15 任务    │
│  预计: 16-20h  已用: 0h                                      │
│                                                             │
│  Phase 2 (核心功能)                                          │
│  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0%   0/13 任务    │
│  预计: 31-37h  已用: 0h                                      │
│                                                             │
│  Phase 3 (高级功能)                                          │
│  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  0%   0/15 任务    │
│  预计: 78-96h  已用: 0h                                      │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  总计: 0/43 任务完成  |  0%  |  已用工时: 0h / 125-153h     │
└─────────────────────────────────────────────────────────────┘
```

---

## 当前焦点

```
📍 当前阶段: Phase 1.1 - 纯静态命令

🎯 下一步任务:
  1. [ ] `/help` - 添加静态帮助文本
  2. [ ] `/commands` - 遍历已注册命令
  3. [ ] `/whoami` - 显示用户 ID
  4. [ ] `/skills` - 列出可用技能

⏱️ 预计完成时间: 2.25 小时
```

---

## 架构决策点

### ADR-001: 是否引入 Session 抽象？

**背景**: 当前 Agent 是无状态的，history 在每次请求后丢弃。

**选项**:

| 选项 | 优点 | 缺点 |
|------|------|------|
| **引入 Session** | 支持多用户并发、会话持久化、上下文保持 | 架构复杂度增加、内存占用 |
| **保持无状态** | 简单、低内存 | 无法支持会话管理、功能受限 |

**推荐**: ✅ **引入 Session**

**理由**:
1. 会话管理是 Phase 2+ 的前置条件
2. 多用户场景必需
3. 与 OpenClaw 功能对齐

### ADR-002: Session 存储策略

**选项**:

| 选项 | 适用场景 | 实现复杂度 |
|------|----------|------------|
| **内存存储** | 单实例、低并发 | 低 |
| **Redis** | 多实例、高并发 | 中 |
| **SQLite** | 单实例、持久化需求 | 中 |

**推荐**: ✅ **内存存储 + SQLite 持久化**

**理由**:
1. 默认内存存储满足大部分场景
2. 可选 SQLite 持久化应对重启
3. 避免引入 Redis 依赖

### ADR-003: 全局状态管理策略

**背景**: Slash command handler 需要访问 Agent 状态。

**选项**:

| 选项 | 实现方式 | 优缺点 |
|------|----------|--------|
| **静态全局变量** | `OnceLock<RwLock<State>>` | 简单，但测试困难 |
| **依赖注入** | 在 Channel 创建时传入 | 测试友好，但改动大 |

**推荐**: ✅ **静态全局变量 (分阶段)**

**理由**:
1. 快速实现 Phase 1
2. Phase 2 可重构为依赖注入
3. 避免过度设计

---

## 风险评估

### 技术风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| Session 并发问题 | 中 | 高 | 使用 RwLock + 分片锁 |
| 内存泄漏 | 低 | 高 | 后台清理过期 session |
| Config 热更新冲突 | 低 | 中 | 版本号 + CAS 写入 |
| Provider 不支持 extended thinking | 高 | 低 | 降级为普通模式 |

### 范围风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 功能蔓延 | 高 | 中 | 严格按 Phase 执行 |
| OpenClaw 功能差距 | 中 | 低 | 明确功能边界 |
| 性能回归 | 低 | 高 | 基准测试 |

### 维护风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 测试覆盖不足 | 中 | 高 | 每个 Phase 添加测试 |
| 文档缺失 | 中 | 低 | 同步更新 docs/ |

---

## 附录

### A. 相关文件清单

```
新增文件:
├── src/agent/state.rs          # AgentStateSnapshot
├── src/session/mod.rs          # Session 抽象
├── src/session/manager.rs      # SessionManager
├── src/session/cleanup.rs      # 过期清理
├── src/session/compaction.rs   # /compact 实现
├── src/config/runtime.rs       # RuntimeConfig
├── src/config/hot_reload.rs    # 配置热更新
├── src/approval/mod.rs         # 审批系统
└── docs/slash-commands.md      # 用户文档

修改文件:
├── src/channels/discord_slash.rs  # 添加命令定义
├── src/channels/discord.rs        # 实现 command handler
├── src/agent/agent.rs             # 状态更新 + Session 集成
├── src/config/schema.rs           # OwnerConfig + 新配置项
└── src/tools/mod.rs               # Skill 工具暴露
```

### B. 测试策略

```
单元测试:
├── AgentStateSnapshot 序列化
├── SessionManager 创建/销毁
├── RuntimeConfig 覆盖
└── ApprovalQueue 状态机

集成测试:
├── Slash command → Agent 状态读取
├── Session 重置 → history 清空
├── /model 切换 → Agent 重建
└── /config 修改 → 配置持久化

端到端测试:
├── Discord 用户 → /status 响应
├── 多用户并发 → Session 隔离
└── 配置热更新 → 无重启生效
```

### C. 参考文档

- OpenClaw Slash Commands: `openclaw/docs/tools/slash-commands.md`
- Discord API Reference: https://discord.com/developers/docs/interactions/application-commands
- AGENTS.md: `/Users/jycyun/vibe/amonkert/zeroclaw/AGENTS.md`
- Session 管理笔记: `docs/vibe/notes-details-20260307.md`
