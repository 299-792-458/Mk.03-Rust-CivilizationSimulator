//! Entity component definitions for the TERA simulation.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::simulation::Nation;

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Identity {
    pub id: u64,
    pub name: String,
    pub faction: Faction,
    pub nation: Nation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Neutral,
    MerchantGuild,
    BanditClans,
    ExplorersLeague,
    SettlersUnion,
    TempleOfSuns,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Attributes {
    pub health: f32,
    pub stamina: f32,
    pub wealth: f32,
    pub fame: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Biome {
    Forest,
    Plains,
    Desert,
    Village,
    Market,
}

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub biome: Biome,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<ItemStack>,
    pub currency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: ItemKind,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemKind {
    Resource(String),
    Equipment(String),
    Artifact(String),
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Personality {
    pub aggressive: f32,
    pub cautious: f32,
    pub social: f32,
    pub curious: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GoalKind {
    Wealth,
    Glory,
    Survival,
    Influence,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Goals {
    pub primary: GoalKind,
    pub intensity: f32, // 0..1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorState {
    Idle,
    Explore,
    Gather,
    Trade,
    Hunt,
    Rest,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]

pub struct Behavior {
    pub state: BehaviorState,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]

pub struct InCombat {
    pub ticks_remaining: u32,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]

pub struct Combatants {
    pub nation_a: Nation,

    pub nation_b: Nation,
}
