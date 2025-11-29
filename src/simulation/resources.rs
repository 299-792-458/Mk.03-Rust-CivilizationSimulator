//! Shared resources and world-level data structures.

use std::time::Duration;

use crate::simulation::Nation;
use crate::simulation::{Era, Tech, WeaponTier};
use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct NationMetrics {
    pub economy: f32,   // 경제
    pub science: f32,   // 과학
    pub culture: f32,   // 문화
    pub diplomacy: f32, // 외교
    pub religion: f32,  // 종교
    pub military: f32,
    pub territory: f32,
    pub is_destroyed: bool,
    pub era: Era,
    pub weapon_tier: WeaponTier,
    pub unlocked_techs: Vec<Tech>,
    pub research_stock: f32,
    pub culture_stock: f32,
    pub population: u64,
    pub youth: u64,
    pub adult: u64,
    pub elder: u64,
    pub productivity: f32,
    pub unemployment: f32,
}

impl Default for NationMetrics {
    fn default() -> Self {
        Self {
            economy: 50.0,
            science: 20.0,
            culture: 30.0,
            diplomacy: 30.0,
            religion: 25.0,
            military: 20.0,
            territory: 33.33,
            is_destroyed: false,
            era: Era::Dawn,
            weapon_tier: WeaponTier::KnappedStone,
            unlocked_techs: vec![Tech::Knapping],
            research_stock: 0.0,
            culture_stock: 0.0,
            population: 3_000_000,
            youth: 900_000,
            adult: 1_800_000,
            elder: 300_000,
            productivity: 1.0,
            unemployment: 6.0,
        }
    }
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct NationCivState {
    pub cities: u32,
    pub happiness: f32,
    pub stability: f32,
    pub production: f32,
}

impl Default for NationCivState {
    fn default() -> Self {
        Self {
            cities: 2,
            happiness: 65.0,
            stability: 60.0,
            production: 40.0,
        }
    }
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct AllNationCivState(pub HashMap<Nation, NationCivState>);

impl Default for AllNationCivState {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(Nation::Tera, NationCivState::default());
        map.insert(Nation::Sora, NationCivState::default());
        map.insert(Nation::Aqua, NationCivState::default());
        map.insert(Nation::Solar, NationCivState::default());
        map.insert(Nation::Luna, NationCivState::default());
        Self(map)
    }
}

#[derive(Debug, Resource, Clone, serde::Serialize, serde::Deserialize)]
pub struct AllNationMetrics(pub HashMap<Nation, NationMetrics>);

impl Default for AllNationMetrics {
    fn default() -> Self {
        let mut metrics = HashMap::new();
        metrics.insert(Nation::Tera, NationMetrics::default());
        metrics.insert(Nation::Sora, NationMetrics::default());
        metrics.insert(Nation::Aqua, NationMetrics::default());
        metrics.insert(Nation::Solar, NationMetrics::default());
        metrics.insert(Nation::Luna, NationMetrics::default());
        Self(metrics)
    }
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize, Default)]
pub struct NuclearBlasts(pub HashMap<crate::simulation::AxialCoord, u8>);

#[derive(Debug, Clone, Resource, Serialize, Deserialize, Default)]
pub struct WarFatigue {
    pub intensity: f32,
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize, Default)]
pub struct WorldRichness {
    /// Aggregated richness score for TUI overlay (0..1)
    pub richness: f32,
}

/// 전지구 생태/기후 상태
#[derive(Debug, Clone, Resource, Serialize, Deserialize, Default)]
pub struct ClimateState {
    pub carbon_ppm: f32,
    pub climate_risk: f32,
    pub biodiversity: f32,
}

/// 과학 승리(달 탐사) 진행도 추적.
#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct ScienceVictory {
    pub progress: HashMap<Nation, f32>,
    pub goal: f32,
    pub leader_history: Vec<f32>,
    pub milestones: HashMap<Nation, u8>,
    pub finished: bool,
    pub winner: Option<Nation>,
    pub interstellar_mode: bool,
    pub interstellar_progress: f32,
    pub interstellar_goal: f32,
}

impl Default for ScienceVictory {
    fn default() -> Self {
        let mut progress = HashMap::new();
        for nation in [
            Nation::Tera,
            Nation::Sora,
            Nation::Aqua,
            Nation::Solar,
            Nation::Luna,
        ] {
            progress.insert(nation, 0.0);
        }
        Self {
            progress,
            goal: 100.0,
            leader_history: Vec::new(),
            milestones: HashMap::new(),
            finished: false,
            winner: None,
            interstellar_mode: false,
            interstellar_progress: 0.0,
            interstellar_goal: 100.0,
        }
    }
}

#[derive(Debug, Resource)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Clone, Resource)]
pub struct SimulationConfig {
    pub tick_duration: Duration,
    pub grid_radius: i32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_duration: Duration::from_secs(1),
            grid_radius: 12,
        }
    }
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct WorldTime {
    pub tick: u64,
}

impl Default for WorldTime {
    fn default() -> Self {
        Self { tick: 0 }
    }
}
