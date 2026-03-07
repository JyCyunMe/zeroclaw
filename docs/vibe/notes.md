# ZeroClaw Development Notes

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
