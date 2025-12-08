use bevy_ecs::prelude::*;

use crate::simulation::{
    AllNationMetrics, DiplomaticRelations, IdeologyMatrix, Nation, WarFatigue,
};

/// Applies sanctions and alliance protection effects to metrics.
pub fn security_system(
    diplo: Res<DiplomaticRelations>,
    mut metrics: ResMut<AllNationMetrics>,
    war: Res<WarFatigue>,
    mut ideology: ResMut<IdeologyMatrix>,
) {
    // Sanction penalties
    let mut sanction_count: Vec<(Nation, u32)> = Vec::new();
    for (_issuer, target) in diplo.sanctions.iter() {
        sanction_count.push((*target, 1));
    }
    for (nation, penalty_times) in sanction_count {
        if let Some(m) = metrics.0.get_mut(&nation) {
            let factor = (1.0 - 0.08 * penalty_times as f32 - m.trade_penalty * 0.01).max(0.5);
            m.economy *= factor;
            m.trade_penalty += penalty_times as f32 * 2.0;
            m.science *= (1.0 - 0.02 * penalty_times as f32).max(0.7);
        }
    }

    // War fatigue spills into diplomacy strength (reverse via alliances)
    let war_pressure = (war.intensity / 100.0).clamp(0.0, 1.0);
    for (nation, m) in metrics.0.iter_mut() {
        m.diplomacy = (m.diplomacy - war_pressure * 10.0 - m.trade_penalty * 0.2).max(0.0);
        m.trade_penalty *= 0.95;
        if let Some(vol) = ideology.volatility.get_mut(nation) {
            *vol = (*vol + war_pressure * 8.0).clamp(0.0, 140.0);
        }
    }

    // Alliance reassurance gives slight diplomacy boost
    for (a, b) in diplo.alliances.iter() {
        if let Some(ma) = metrics.0.get_mut(a) {
            ma.diplomacy = (ma.diplomacy + 2.0).min(200.0);
        }
        if let Some(mb) = metrics.0.get_mut(b) {
            mb.diplomacy = (mb.diplomacy + 2.0).min(200.0);
        }
        if let Some(coh_a) = ideology.cohesion.get_mut(a) {
            *coh_a = (*coh_a + 1.0).min(120.0);
        }
        if let Some(coh_b) = ideology.cohesion.get_mut(b) {
            *coh_b = (*coh_b + 1.0).min(120.0);
        }
    }
}
