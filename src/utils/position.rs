//! Position utilities.

use pumpkin_plugin_api::world::Position as PumpkinPosition;
use serde::{Deserialize, Serialize};

/// Wrapper around Pumpkin's Position for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl Position {
    pub fn new(world: String, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        Self { world, x, y, z, yaw, pitch }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
}

/// Pumpkin's Position is a tuple (f64, f64, f64) representing (x, y, z).
/// Our Position has additional fields (world, yaw, pitch) for serialization.
impl From<PumpkinPosition> for Position {
    fn from(pos: PumpkinPosition) -> Self {
        // PumpkinPosition is a tuple (f64, f64, f64) = (x, y, z)
        let (x, y, z) = pos;
        Self {
            world: "world".to_string(), // Default world
            x,
            y,
            z,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl From<Position> for PumpkinPosition {
    fn from(pos: Position) -> Self {
        // PumpkinPosition is a tuple (f64, f64, f64) = (x, y, z)
        (pos.x, pos.y, pos.z)
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.world == other.world
            && (self.x - other.x).abs() < f64::EPSILON
            && (self.y - other.y).abs() < f64::EPSILON
            && (self.z - other.z).abs() < f64::EPSILON
    }
}