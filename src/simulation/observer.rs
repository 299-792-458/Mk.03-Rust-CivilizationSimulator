//! Shared observer snapshot structures exported via the API.

use crate::simulation::{
    AllNationCivState, AllNationMetrics, AxialCoord, BehaviorState, Biome, Faction, Nation,
    WorldEvent,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize)]
pub struct EntitySnapshot {
    pub id: u64,
    pub name: String,
    pub faction: Faction,
    pub faction_label: String,
    pub biome: Biome,
    pub biome_label: String,
    pub behavior_state: BehaviorState,
    pub behavior_label: String,
    pub currency: f32,
    pub wealth: f32,
    pub fame: f32,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct HexGridSnapshot {
    pub hexes: HashMap<AxialCoord, HexSnapshot>,
    pub radius: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HexSnapshot {
    pub owner: Nation,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeasonEffectSnapshot {
    pub label: String,
    pub temperature: f32,
    pub morale_shift: f32,
    pub yield_shift: f32,
    pub risk_shift: f32,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct WorldOverlaySnapshot {
    pub war_fatigue: f32,
    pub fallout: f32,
    pub resource_richness: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObserverSnapshot {
    pub tick: u64,
    pub epoch: String,
    pub season: String,
    pub season_effect: SeasonEffectSnapshot,
    pub all_metrics: AllNationMetrics,
    pub civ_state: AllNationCivState,
    pub grid: HexGridSnapshot,
    pub overlay: WorldOverlaySnapshot,
    pub entities: Vec<EntitySnapshot>,
    pub events: Vec<WorldEvent>,
    pub combat_hexes: HashSet<AxialCoord>,
    pub nuclear_hexes: HashSet<AxialCoord>,
}

impl ObserverSnapshot {
    pub fn new() -> Self {
        Self {
            tick: 0,
            epoch: "새벽".to_string(),
            season: "꽃피움 계절".to_string(),
            season_effect: SeasonEffectSnapshot {
                label: "온화한 바람".to_string(),
                temperature: 0.0,
                morale_shift: 0.0,
                yield_shift: 0.0,
                risk_shift: 0.0,
            },
            all_metrics: AllNationMetrics::default(),
            civ_state: AllNationCivState::default(),
            grid: HexGridSnapshot::default(),
            overlay: WorldOverlaySnapshot::default(),
            entities: Vec::new(),
            events: Vec::new(),
            combat_hexes: HashSet::new(),
            nuclear_hexes: HashSet::new(),
        }
    }

    pub fn update(
        &mut self,
        tick: u64,
        epoch: String,
        season: String,
        season_effect: SeasonEffectSnapshot,
        metrics: &AllNationMetrics,
        civ_state: AllNationCivState,
        grid: HexGridSnapshot,
        overlay: WorldOverlaySnapshot,
        entities: Vec<EntitySnapshot>,
        events: Vec<WorldEvent>,
        combat_hexes: HashSet<AxialCoord>,
        nuclear_hexes: HashSet<AxialCoord>,
    ) {
        self.tick = tick;
        self.epoch = epoch;
        self.season = season;
        self.season_effect = season_effect;
        self.all_metrics = metrics.clone();
        self.civ_state = civ_state;
        self.grid = grid;
        self.overlay = overlay;
        self.entities = entities;
        self.events = events;
        self.combat_hexes = combat_hexes;
        self.nuclear_hexes = nuclear_hexes;
    }
}

impl Default for ObserverSnapshot {
    fn default() -> Self {
        Self::new()
    }
}
