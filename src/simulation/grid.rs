use bevy_ecs::prelude::{Component, Entity, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::simulation::Nation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct AxialCoord {
    pub q: i32,
    pub r: i32,
}

impl AxialCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn neighbors(&self) -> [AxialCoord; 6] {
        [
            AxialCoord::new(self.q + 1, self.r),
            AxialCoord::new(self.q - 1, self.r),
            AxialCoord::new(self.q, self.r + 1),
            AxialCoord::new(self.q, self.r - 1),
            AxialCoord::new(self.q + 1, self.r - 1),
            AxialCoord::new(self.q - 1, self.r + 1),
        ]
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Hex {
    pub owner: Nation,
    pub elevation: f32,
    pub biome: crate::simulation::Biome,
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize, Default)]
pub struct HexGrid {
    pub hexes: HashMap<AxialCoord, Entity>,
    pub radius: i32,
}

#[allow(dead_code)]
pub fn default_continent_centers(radius: i32) -> [(Nation, AxialCoord); 3] {
    [
        (Nation::Tera, AxialCoord::new(-radius + 3, radius / 3)),
        (Nation::Sora, AxialCoord::new(radius - 3, -radius / 4)),
        (Nation::Aqua, AxialCoord::new(0, -radius + 3)),
    ]
}

pub fn pentagon_centers(radius: i32) -> [(Nation, AxialCoord); 5] {
    let r = radius - 2;
    [
        (Nation::Tera, AxialCoord::new(0, -r)),     // top
        (Nation::Sora, AxialCoord::new(r, -r / 2)), // upper right
        (Nation::Aqua, AxialCoord::new(r, r / 2)),  // lower right
        (Nation::Solar, AxialCoord::new(0, r)),     // bottom
        (Nation::Luna, AxialCoord::new(-r, 0)),     // left
    ]
}
