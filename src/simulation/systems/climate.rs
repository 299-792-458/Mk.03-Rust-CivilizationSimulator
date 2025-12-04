use bevy_ecs::prelude::*;

use crate::simulation::{
    ClimateState, NuclearBlasts, WarFatigue, WorldEvent, WorldEventKind, WorldEventLog,
    WorldRichness, WorldTime,
};

/// Updates global climate state based on war/industry signals.
pub fn climate_system(
    mut climate: ResMut<ClimateState>,
    richness: Res<WorldRichness>,
    blasts: Res<NuclearBlasts>,
    fatigue: Res<WarFatigue>,
    cosmic: Res<crate::simulation::CosmicTimeline>,
    time: Res<WorldTime>,
    mut log: ResMut<WorldEventLog>,
) {
    // Baseline slow increase
    climate.carbon_ppm += 0.05;

    // Geologic stage scaling
    let stage_factor = match cosmic.geologic_stage.as_str() {
        "원시 지각" => 0.5,
        "태고 해양" => 0.7,
        "산소 폭발" => 1.1,
        "캄브리아/대륙 분화" => 1.2,
        "대멸종 순환" => 1.4,
        _ => 1.6,
    };

    // Industry proxy: richness acts as prosperity => higher emissions
    climate.carbon_ppm += richness.richness * 0.8 * stage_factor;

    // War proxy: war fatigue spikes emissions and risk
    climate.carbon_ppm += fatigue.intensity * 0.02 * stage_factor;

    // Nuclear fallout amplifies climate risk
    let blast_total: u32 = blasts.0.values().map(|v| *v as u32).sum();
    climate.carbon_ppm += blast_total as f32 * 0.03;

    // Biodiversity erosion from carbon/risk
    climate.biodiversity -= (climate.carbon_ppm / 1000.0) * 0.2 * stage_factor;
    climate.biodiversity = climate.biodiversity.max(0.0);

    // Climate risk is a composite
    climate.climate_risk =
        (climate.carbon_ppm * 0.6 + fatigue.intensity * 0.4 + blast_total as f32 * 0.5) * 0.01;
    climate.climate_risk = climate.climate_risk.clamp(0.0, 100.0);
    // Sea level and ice line (normalized 0..1)
    climate.sea_level = (climate.climate_risk * 0.01 * 0.6 + richness.richness * 0.4)
        .min(1.0)
        .max(0.0);
    climate.ice_line = (1.0 - (climate.carbon_ppm / 800.0).min(1.0)) as f32;

    let carbon_ppm = climate.carbon_ppm;
    let risk = climate.climate_risk;
    let bio = climate.biodiversity;
    push_history(&mut climate.carbon_history, carbon_ppm);
    push_history(&mut climate.climate_risk_history, risk);
    push_history(&mut climate.biodiversity_history, bio);

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
    if history.len() > 512 {
        let mut i = 0;
        history.retain(|_| {
            let keep = i % 2 == 0;
            i += 1;
            keep
        });
    }
}
