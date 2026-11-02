# atrium-emotion

[![crates.io](https://img.shields.io/crates/v/atrium-emotion.svg)](https://crates.io/crates/atrium-emotion)
[![docs.rs](https://docs.rs/atrium-emotion/badge.svg)](https://docs.rs/atrium-emotion)
[![CI](https://github.com/chinokoyuki/atrium-emotion/actions/workflows/ci.yml/badge.svg)](https://github.com/chinokoyuki/atrium-emotion/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[English](README.md)

一个小型情感计算引擎。

维护一个三维情感状态（愉悦–唤醒–支配，即 Mehrabian & Russell 的 PAD 模型），并让它按经得起推敲的方式运动：空闲时随机漂移、随昼夜节律起伏、情绪持续时会粘滞，最后把状态归类为离散标签——11 种基本情绪、22 种复合情绪。

不依赖 LLM、数据库、异步运行时。只有 f32 运算、156 个单元测试和一套 criterion 基准。

## 适用场景

- 游戏 NPC：需要一个真实的心境，而不是一个 mood 标志位
- 聊天 Agent：语气应随历史累积变化，而不是每轮清零
- 社交机器人、互动叙事、情感计算原型

## 安装

```toml
[dependencies]
atrium-emotion = "0.1"
```

需要 Rust 1.82+。

## 60 秒上手

```rust
use atrium_emotion::{
    classify_compound, CircadianModulator, CompoundContext, DriftParams, EmotionDirection,
    EmotionEngine, EmotionState, EmotionalInertia, ReunionBurst,
};

let mut engine = EmotionEngine::new(EmotionState::new(0.0, 0.0, 0.0), 0.05)
    .with_drift(DriftParams::new(0.02, 0.01))          // 空闲扰动，有界
    .with_circadian(CircadianModulator::default())     // 上午 / 傍晚双峰
    .with_inertia(EmotionalInertia::default())         // 情绪粘滞
    .with_reunion_burst(ReunionBurst::default());      // 离开后回归的响应

// 事件推动状态。
engine.affect(&EmotionState::new(0.6, 0.4, 0.2));

// 每个 tick：衰减 → 漂移 → 节律偏移 → 惯性更新。
engine.tick_with_hour(14);

// 读取状态。
let label = engine.current_label();
println!("{} {} ({})", label.emoji, label.name, label.en_name);

// 复合情绪分类，叠加方向性。
let ctx = CompoundContext {
    direction: EmotionDirection::UserDirected,
    ..Default::default()
};
if let Some(ce) = classify_compound(engine.current(), &ctx) {
    println!("compound: {} ({})", ce.name, ce.en_name);
}
```

完整可运行版本见 [`examples/quickstart.rs`](examples/quickstart.rs)（`cargo run --example quickstart`）。

## 工作原理

状态是 PAD 空间中的一个点，每轴限制在 [-1, 1]。每次 `tick()` 依次执行：

1. **衰减** — 向默认状态线性插值回归
2. **OU 漂移** — Ornstein-Uhlenbeck 过程叠加有界、均值回归的噪声，空闲状态也会自然波动
3. **昼夜节律** — 双峰高斯（上午 + 傍晚，均可配置）按时段调整唤醒度和愉悦度
4. **情绪惯性** — 若某一标签在近期历史中占比 ≥60%，敏感度升高（最高 1.5×）、衰减变慢（最低 0.85×）；情绪多样化则重置
5. **想念漂移** — 启用后，随离开时长增长，漂移基线从中性向分离基线插值

事件通过 `affect()` 传入 PAD 增量，受当前惯性敏感度缩放。

**分类**是对标签中心点的加权最近邻（arousal 权重 ×1.2），并有中性区间预判，避免近零状态被误判。**复合情绪**在其上叠加：每种复合情绪有 PAD 中心、半径和方向约束（自身 / 对方 / 记忆 / 无方向），因此"内疚"和"感激"即使 PAD 区域相邻也不会混淆。

## 模块

| 模块 | 功能 |
|---|---|
| `EmotionEngine` | 核心循环：衰减 + 漂移 + 节律 + 惯性，快照/恢复 |
| `compound` | 22 种复合情绪、方向感知分类、混合情绪检测 |
| `circadian` | 双峰昼夜节律 |
| `longing` | 分离强度随时间演化、基线插值、表达门控 |
| `reunion` | 回归响应、关系阶段门控、情境变体 |
| `anticipation_preloader` | 向约定事件渐进增强的期待 |
| `disappointment_handler` | 期待过期的失落响应，连续失约递增 |
| `longing_expression_channel` | 节流后的主动表达（阈值 / 冷却 / 会话上限） |
| `longing_narrative_bridge` | 将等待时长转写为叙事文本 |

## 说明

- 情绪名同时提供中英文（`name` / `en_name`）。内置建议用语为中文；数值输出（强度、PAD 偏移）与语言无关，其他语言请自备用语表。
- 所有状态类型实现 `Serialize`/`Deserialize`；`snapshot()` / `restore()` 负责持久化。
- 基准测试：`cargo bench`。完整引擎 tick 在亚微秒量级，放进逐帧游戏循环也没有压力。

## 许可

MIT
