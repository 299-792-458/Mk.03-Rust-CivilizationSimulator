use bevy_ecs::prelude::*;

use crate::simulation::{
    ClimateState, NuclearBlasts, WarFatigue, WorldEvent, WorldEventKind, WorldEventLog, WorldTime,
    WorldRichness,
};

/// Updates global climate state based on war/industry signals.
pub fn climate_system(
    mut climate: ResMut<ClimateState>,
    richness: Res<WorldRichness>,
    blasts: Res<NuclearBlasts>,
    fatigue: Res<WarFatigue>,
    time: Res<WorldTime>,
    mut log: ResMut<WorldEventLog>,
) {
    // Baseline slow increase
    climate.carbon_ppm += 0.05;

    // Industry proxy: richness acts as prosperity => higher emissions
    climate.carbon_ppm += richness.richness * 0.8;

    // War proxy: war fatigue spikes emissions and risk
    climate.carbon_ppm += fatigue.intensity * 0.02;

    // Nuclear fallout amplifies climate risk
    let blast_total: u32 = blasts.0.values().map(|v| *v as u32).sum();
    climate.carbon_ppm += blast_total as f32 * 0.03;

    // Biodiversity erosion from carbon/risk
    climate.biodiversity -= (climate.carbon_ppm / 1000.0) * 0.2;
    climate.biodiversity = climate.biodiversity.max(0.0);

    // Climate risk is a composite
    climate.climate_risk =
        (climate.carbon_ppm * 0.6 + fatigue.intensity * 0.4 + blast_total as f32 * 0.5) * 0.01;
    climate.climate_risk = climate.climate_risk.clamp(0.0, 100.0);
    push_history(&mut climate.carbon_history, climate.carbon_ppm);
    push_history(&mut climate.climate_risk_history, climate.climate_risk);
    push_history(&mut climate.biodiversity_history, climate.biodiversity);

    // Event pulses occasionally
    if time.tick % 24 == 0 {
        log.push(WorldEvent {
            tick: time.tick,
            epoch: "기후".to_string(),
            season: "지구".to_string(),
            kind: WorldEventKind::MacroShock {
                stressor: "기후 변동 경고".to_string(),
                catalyst: format!(
                    "탄소 {:.0}ppm | 위험 {:.1}%",
                    climate.carbon_ppm, climate.climate_risk
                ),
                projected_impact: "생산성 저하·인구 피해 가능".to_string(),
                casualties: None,
            },
        });
    }
}

fn push_history(history: &mut Vec<f32>, value: f32) {
    history.push(value);
    if history.len() > 256 {
        history.remove(0);
    }
}
