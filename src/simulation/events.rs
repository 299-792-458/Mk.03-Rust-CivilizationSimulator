//! Structured world event data and observer-facing snapshots.

use std::collections::VecDeque;

use crate::simulation::Nation;
use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::simulation::{BehaviorState, Biome, Era, Faction, WeaponTier};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldEventKind {
    Trade {
        actor: EventActor,
        trade_focus: String,
        market_pressure: String,
    },
    Social {
        convener: EventActor,
        gathering_theme: String,
        cohesion_level: String,
    },
    MacroShock {
        stressor: String,
        catalyst: String,
        projected_impact: String,
        casualties: Option<u64>,
    },
    Warfare {
        winner: Nation,
        loser: Nation,
        territory_change: f32,
        casualties: u64,
        nuclear: bool,
    },
    EraShift {
        nation: Nation,
        era: Era,
        weapon: WeaponTier,
    },
    ScienceProgress {
        nation: Nation,
        progress: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sentiment {
    Positive,
    Neutral,
    Negative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventActor {
    pub id: u64,
    pub name: String,
    pub nation: crate::simulation::Nation,
    pub faction: Faction,
    pub faction_label: String,
    pub biome: Biome,
    pub biome_label: String,
    pub behavior_hint: BehaviorState,
    pub behavior_hint_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub tick: u64,
    pub epoch: String,
    pub season: String,
    pub kind: WorldEventKind,
}

impl WorldEvent {
    pub fn category(&self) -> &'static str {
        match &self.kind {
            WorldEventKind::Trade { .. } => "무역",
            WorldEventKind::Social { .. } => "사회",
            WorldEventKind::MacroShock { .. } => "거시충격",
            WorldEventKind::Warfare { .. } => "전쟁",
            WorldEventKind::EraShift { .. } => "시대",
            WorldEventKind::ScienceProgress { .. } => "과학",
        }
    }

    pub fn sentiment(&self) -> Sentiment {
        match &self.kind {
            WorldEventKind::Trade { .. } => Sentiment::Positive,
            WorldEventKind::Social { .. } => Sentiment::Positive,
            WorldEventKind::MacroShock { .. } => Sentiment::Negative,
            WorldEventKind::Warfare { .. } => Sentiment::Negative,
            WorldEventKind::EraShift { .. } => Sentiment::Positive,
            WorldEventKind::ScienceProgress { .. } => Sentiment::Positive,
        }
    }

    #[allow(dead_code)]
    pub fn headline(&self) -> String {
        match &self.kind {
            WorldEventKind::Trade {
                actor,
                trade_focus,
                market_pressure,
            } => format!(
                "{} 님이 {} 거래를 조율합니다 | 압력: {}",
                actor.name, trade_focus, market_pressure
            ),
            WorldEventKind::Social {
                convener,
                gathering_theme,
                cohesion_level,
            } => format!(
                "{} 님이 \"{}\" 주제로 모임을 주관합니다 | 응집도: {}",
                convener.name, gathering_theme, cohesion_level
            ),
            WorldEventKind::MacroShock {
                stressor,
                catalyst,
                projected_impact,
                casualties,
            } => format!(
                "{} | 촉발 요인: {} | 영향: {}{}",
                stressor,
                catalyst,
                projected_impact,
                casualties
                    .map(|c| format!(" | 피해: {}", crate::simulation::format_number_commas(c)))
                    .unwrap_or_default()
            ),
            WorldEventKind::Warfare {
                winner,
                loser,
                territory_change,
                casualties,
                nuclear,
            } => format!(
                "{}가 {}와의 전쟁에서 승리하여 영토 {:.2}를 획득했습니다. 사상자 {}명{}",
                winner.name(),
                loser.name(),
                territory_change,
                crate::simulation::format_number_commas(*casualties),
                if *nuclear { " | 핵 공격" } else { "" }
            ),
            WorldEventKind::EraShift {
                nation,
                era,
                weapon,
            } => format!(
                "{}가 {}에 돌입했습니다 | 주력 무기: {}",
                nation.name(),
                era.label(),
                weapon.label()
            ),
            WorldEventKind::ScienceProgress { nation, progress } => format!(
                "{}의 달 탐사 진행 {:.1}% / 100% (1틱=1세대)",
                nation.name(),
                progress.min(100.0)
            ),
        }
    }

    pub fn trade(
        tick: u64,
        epoch: &str,
        season: &str,
        actor: EventActor,
        trade_focus: String,
        market_pressure: String,
    ) -> Self {
        Self {
            tick,
            epoch: epoch.to_string(),
            season: season.to_string(),
            kind: WorldEventKind::Trade {
                actor,
                trade_focus,
                market_pressure,
            },
        }
    }

    pub fn social(
        tick: u64,
        epoch: &str,
        season: &str,
        convener: EventActor,
        gathering_theme: String,
        cohesion_level: String,
    ) -> Self {
        Self {
            tick,
            epoch: epoch.to_string(),
            season: season.to_string(),
            kind: WorldEventKind::Social {
                convener,
                gathering_theme,
                cohesion_level,
            },
        }
    }

    pub fn macro_shock(
        tick: u64,
        epoch: &str,
        season: &str,
        stressor: String,
        catalyst: String,
        projected_impact: String,
        casualties: Option<u64>,
    ) -> Self {
        Self {
            tick,
            epoch: epoch.to_string(),
            season: season.to_string(),
            kind: WorldEventKind::MacroShock {
                stressor,
                catalyst,
                projected_impact,
                casualties,
            },
        }
    }

    pub fn warfare(
        tick: u64,
        epoch: &str,
        season: &str,
        winner: Nation,
        loser: Nation,
        territory_change: f32,
        casualties: u64,
        nuclear: bool,
    ) -> Self {
        Self {
            tick,
            epoch: epoch.to_string(),
            season: season.to_string(),
            kind: WorldEventKind::Warfare {
                winner,
                loser,
                territory_change,
                casualties,
                nuclear,
            },
        }
    }

    pub fn era_shift(
        tick: u64,
        epoch: &str,
        season: &str,
        nation: Nation,
        era: Era,
        weapon: WeaponTier,
    ) -> Self {
        Self {
            tick,
            epoch: epoch.to_string(),
            season: season.to_string(),
            kind: WorldEventKind::EraShift {
                nation,
                era,
                weapon,
            },
        }
    }

    pub fn science_progress(
        tick: u64,
        epoch: &str,
        season: &str,
        nation: Nation,
        progress: f32,
    ) -> Self {
        Self {
            tick,
            epoch: epoch.to_string(),
            season: season.to_string(),
            kind: WorldEventKind::ScienceProgress { nation, progress },
        }
    }
}

#[derive(Debug, Resource)]
pub struct WorldEventLog {
    events: VecDeque<WorldEvent>,
    capacity: usize,
}

impl WorldEventLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, event: WorldEvent) {
        if self.events.len() == self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn snapshot(&self) -> Vec<WorldEvent> {
        self.events.iter().cloned().collect()
    }
}

impl Default for WorldEventLog {
    fn default() -> Self {
        Self::new(256)
    }
}
