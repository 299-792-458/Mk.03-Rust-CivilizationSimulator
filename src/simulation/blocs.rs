use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::simulation::Nation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BlocKind {
    ResearchPact,
    Sanction,
    DefenseTreaty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bloc {
    pub kind: BlocKind,
    pub members: HashSet<Nation>,
    pub leader: Option<Nation>,
    pub strength: f32,
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize, Default)]
pub struct WorldBlocs {
    pub blocs: HashMap<BlocKind, Bloc>,
}
