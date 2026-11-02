// SPDX-License-Identifier: MIT
//! Quickstart — a short tour of the engine.

use atrium_emotion::{
    classify_compound, CircadianModulator, CompoundContext, DriftParams, EmotionDirection,
    EmotionEngine, EmotionState, EmotionalInertia, ReunionBurst,
};

fn main() {
    // 1. Build an engine: default PAD state + decay rate, then stack features.
    let mut engine = EmotionEngine::new(EmotionState::new(0.0, 0.0, 0.0), 0.05)
        .with_drift(DriftParams::new(0.02, 0.01)) // idle wobble around the baseline
        .with_circadian(CircadianModulator::default()) // morning / evening energy peaks
        .with_inertia(EmotionalInertia::default()) // moods get sticky over time
        .with_reunion_burst(ReunionBurst::default()); // scale joy on return

    // 2. Events push the PAD state around.
    engine.affect(&EmotionState::new(0.6, 0.4, 0.2)); // something nice happened
    let state = engine.current();
    println!(
        "PAD now: ({:.2}, {:.2}, {:.2})",
        state.pleasure, state.arousal, state.dominance
    );
    let label = engine.current_label();
    println!("label: {} {} ({})", label.emoji, label.name, label.en_name);

    // 3. Compound classification adds direction-aware nuance on top.
    let ctx = CompoundContext {
        direction: EmotionDirection::UserDirected,
        ..Default::default()
    };
    if let Some(ce) = classify_compound(engine.current(), &ctx) {
        println!("compound: {} {} ({})", ce.emoji, ce.name, ce.en_name);
    }

    // 4. Each tick decays the state toward the default, then applies
    //    drift, circadian offset, and inertia updates.
    for hour in 0..24 {
        engine.tick_with_hour(hour);
    }
    let state = engine.current();
    println!(
        "after a day of ticks: ({:.2}, {:.2}, {:.2})",
        state.pleasure, state.arousal, state.dominance
    );

    // 5. Someone was away for an hour and comes back.
    if let Some(expr) = engine.on_reunion(3600, 0.6) {
        println!("reunion intensity: {:.2}", expr.intensity);
        println!("suggested phrase: {}", expr.suggested_phrases[0]);
    }

    // 6. Snapshot for persistence (serde Serialize/Deserialize).
    let snap = engine.snapshot();
    let json = serde_json::to_string(&snap).expect("snapshot serializes");
    println!("snapshot JSON: {} bytes", json.len());
}
