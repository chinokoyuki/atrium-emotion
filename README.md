# atrium-emotion

[![crates.io](https://img.shields.io/crates/v/atrium-emotion.svg)](https://crates.io/crates/atrium-emotion)
[![docs.rs](https://docs.rs/atrium-emotion/badge.svg)](https://docs.rs/atrium-emotion)
[![CI](https://github.com/chinokoyuki/atrium-emotion/actions/workflows/ci.yml/badge.svg)](https://github.com/chinokoyuki/atrium-emotion/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[中文说明](README_ZH.md)

A small affective-computing engine for Rust.

It maintains a 3-dimensional emotional state (Pleasure–Arousal–Dominance, after Mehrabian & Russell) and moves it around in ways that hold up to scrutiny: it drifts when idle, follows a daily rhythm, gets sticky when a mood lingers, and classifies the result into discrete labels — 11 basic ones, 22 compound ones.

No LLM, no database, no async runtime. Just f32 math, 156 unit tests, and a criterion bench suite.

## Where this fits

- Game NPCs that need an actual mood instead of a mood flag
- Chat agents whose tone should track history instead of resetting every turn
- Social robots, interactive fiction, affective-computing prototypes

## Install

```toml
[dependencies]
atrium-emotion = "0.1"
```

Requires Rust 1.82+.

## The 60-second tour

```rust
use atrium_emotion::{
    classify_compound, CircadianModulator, CompoundContext, DriftParams, EmotionDirection,
    EmotionEngine, EmotionState, EmotionalInertia, ReunionBurst,
};

let mut engine = EmotionEngine::new(EmotionState::new(0.0, 0.0, 0.0), 0.05)
    .with_drift(DriftParams::new(0.02, 0.01))          // idle wobble, bounded
    .with_circadian(CircadianModulator::default())     // morning / evening peaks
    .with_inertia(EmotionalInertia::default())         // moods get sticky
    .with_reunion_burst(ReunionBurst::default());      // return-after-absence response

// Events push the state around.
engine.affect(&EmotionState::new(0.6, 0.4, 0.2));

// Every tick: decay → drift → circadian offset → inertia update.
engine.tick_with_hour(14);

// Read the state.
let label = engine.current_label();
println!("{} {} ({})", label.emoji, label.name, label.en_name);

// Compound classification adds direction-aware nuance.
let ctx = CompoundContext {
    direction: EmotionDirection::UserDirected,
    ..Default::default()
};
if let Some(ce) = classify_compound(engine.current(), &ctx) {
    println!("compound: {} ({})", ce.name, ce.en_name);
}
```

Full runnable version: [`examples/quickstart.rs`](examples/quickstart.rs) (`cargo run --example quickstart`).

## How it works

State is a point in PAD space, each axis clamped to [-1, 1]. Each `tick()` applies, in order:

1. **Decay** — linear interpolation back toward the default state
2. **OU drift** — an Ornstein-Uhlenbeck process adds bounded, mean-reverting noise, so idle states still wobble
3. **Circadian offset** — a bimodal Gaussian (morning + evening peaks, both configurable) shifts arousal and pleasure by time of day
4. **Inertia** — if one label dominates ≥60% of recent history, sensitivity rises (up to 1.5×) and decay slows (down to 0.85×); variety resets it
5. **Longing drift** — if enabled, the baseline interpolates from neutral toward a separation baseline as absence time grows

Events call `affect()` with a PAD delta, scaled by current inertia sensitivity.

**Classification** is weighted nearest-neighbor over label centroids (arousal weighted ×1.2), with a neutral-zone precheck so near-zero states don't get mislabeled. **Compound emotions** layer on top: each has a PAD center, a radius, and a direction constraint (self / other / memory / neutral), so "guilt" and "gratitude" live in adjacent PAD regions but never collide.

## Modules

| Module | What it does |
|---|---|
| `EmotionEngine` | core loop: decay + drift + circadian + inertia, snapshot/restore |
| `compound` | 22 compound emotions, direction-aware classification, mixed-emotion detection |
| `circadian` | bimodal daily rhythm |
| `longing` | separation intensity over time, baseline interpolation, expression gating |
| `reunion` | return-after-absence responses, relationship-stage gating, context variants |
| `anticipation_preloader` | gradual ramp-up toward a scheduled event |
| `disappointment_handler` | missed-appointment response with consecutive escalation |
| `longing_expression_channel` | throttled proactive expression (threshold / cooldown / session cap) |
| `longing_narrative_bridge` | converts waiting time into narrative text |

## Notes

- Emotion names ship in both Chinese and English (`name` / `en_name`). Built-in suggested phrases are Chinese; the numeric outputs (intensity, PAD offsets) are language-neutral, so bring your own phrase tables for other languages.
- All state types derive `Serialize`/`Deserialize`; `snapshot()` / `restore()` cover persistence.
- Benchmarks: `cargo bench`. The full engine tick is sub-microsecond territory — cheap enough for a per-frame game loop.

## License

MIT
