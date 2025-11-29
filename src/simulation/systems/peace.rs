use bevy_ecs::prelude::*;

use crate::simulation::{AllNationMetrics, ScienceVictory, WarFatigue};

/// Soft recovery system rewarding 평화 상태: 낮은 전쟁 피로에서 경제/문화/과학이 회복.
pub fn peace_recovery_system(
    mut metrics: ResMut<AllNationMetrics>,
    warfatigue: Res<WarFatigue>,
    science_victory: Res<ScienceVictory>,
) {
    if science_victory.finished {
        return;
    }

    let fatigue = warfatigue.intensity;
    if fatigue > 40.0 {
        return;
    }

    let recovery = (40.0 - fatigue).max(0.0) * 0.04;
    for (_, m) in metrics.0.iter_mut() {
        if m.is_destroyed {
            continue;
        }
        m.economy = (m.economy + recovery * 0.35).min(120.0);
        m.culture = (m.culture + recovery * 0.25).min(120.0);
        m.science = (m.science + recovery * 0.3).min(150.0);
        m.research_stock += recovery * 0.12;
    }
}
