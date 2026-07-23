use crate::{GRID_SIZE, PLAYER_SIZE};
use bevy::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::time::Instant;

#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct DebugMode(pub bool);

#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct TurnCounter(pub u32);
#[derive(Component, Clone, Default, Debug)]
pub struct TurnCountText;

#[derive(Component, Clone, Default, Debug)]
pub struct Player;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum PlayerAction {
    RollForward,
    RollBackward,
    RollLeft,
    RollRight,
    TurnLeft,
    TurnRight,
    TurnAround,
    SlideLeft,
    SlideRight,
    Wait,
}
impl PlayerAction {
    pub const ALL: [Self; 10] = [
        Self::RollForward,
        Self::RollBackward,
        Self::RollLeft,
        Self::RollRight,
        Self::TurnLeft,
        Self::TurnRight,
        Self::TurnAround,
        Self::SlideLeft,
        Self::SlideRight,
        Self::Wait,
    ];

    pub fn key_code(self) -> KeyCode {
        match self {
            Self::RollForward => KeyCode::KeyW,
            Self::RollBackward => KeyCode::KeyS,
            Self::RollLeft => KeyCode::KeyA,
            Self::RollRight => KeyCode::KeyD,
            Self::TurnLeft => KeyCode::KeyQ,
            Self::TurnRight => KeyCode::KeyE,
            Self::TurnAround => KeyCode::KeyX,
            Self::SlideLeft => KeyCode::KeyZ,
            Self::SlideRight => KeyCode::KeyC,
            Self::Wait => KeyCode::Space,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct AvailableActions(HashSet<PlayerAction>);
impl Default for AvailableActions {
    fn default() -> Self {
        Self(PlayerAction::ALL.into_iter().collect())
    }
}
impl AvailableActions {
    pub fn contains(&self, action: PlayerAction) -> bool {
        self.0.contains(&action)
    }

    pub fn toggle(&mut self, action: PlayerAction) -> bool {
        if self.0.remove(&action) {
            false
        } else {
            self.0.insert(action);
            true
        }
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct Arrow;
#[derive(Component, Clone, Default, Debug)]
pub struct CameraRig;

#[derive(Component, Clone, Default, Debug)]
pub struct GridLocation(pub Vec3);
impl GridLocation {
    pub fn to_world_space(&self) -> Vec3 {
        let mut grid_location = vec3(self.0.x, 0.0, self.0.z);
        grid_location *= GRID_SIZE.extend(GRID_SIZE.y);
        grid_location
    }
}

#[derive(Component, Clone, Debug)]
pub struct Moving {
    pub direction: Direction,
    pub start: Instant,
    pub initial_rotation: Quat,
}
#[derive(Component, Clone, Default, Debug)]
pub struct Orientation(pub Direction);
#[derive(Message, Clone, Debug)]
pub struct CompletedTurn;

#[derive(Copy, Clone, Default, Debug)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
    Left,
    Right,
    Around,
    SlideLeft,
    SlideRight,
    Wait,
}
impl Direction {
    pub fn turn_left(self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            Direction::Left => Direction::Around, // the last 3 should never be called, but i cant be bothered to make sure they aren't
            Direction::Right => Direction::Around,
            Direction::Around => Direction::Left,
            Direction::SlideLeft => Direction::SlideLeft,
            Direction::SlideRight => Direction::SlideRight,
            Direction::Wait => Direction::Wait,
        }
    }
    pub fn turn_right(self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            Direction::Left => Direction::Around, // the last 3 should never be called, but i cant be bothered to make sure they aren't
            Direction::Right => Direction::Around,
            Direction::Around => Direction::Right,
            Direction::SlideLeft => Direction::SlideLeft,
            Direction::SlideRight => Direction::SlideRight,
            Direction::Wait => Direction::Wait,
        }
    }
    pub fn to_rotation(self) -> Quat {
        match self {
            Direction::North => Quat::from_rotation_y(0.0),
            Direction::West => Quat::from_rotation_y(PI / 2.0),
            Direction::South => Quat::from_rotation_y(PI),
            Direction::East => Quat::from_rotation_y(3.0 * PI / 2.0),
            Direction::Left => {
                error!("Left does not have a rotation");
                Quat::from_rotation_y(0.0)
            }
            Direction::Right => {
                error!("Right does not have a rotation");
                Quat::from_rotation_y(0.0)
            }
            Direction::Around => {
                error!("Around does not have a rotation");
                Quat::from_rotation_y(0.0)
            }
            Direction::SlideLeft => {
                error!("SlideLeft does not have a rotation");
                Quat::IDENTITY
            }
            Direction::SlideRight => {
                error!("SlideRight does not have a rotation");
                Quat::IDENTITY
            }
            Direction::Wait => {
                error!("Wait does not have a rotation");
                Quat::IDENTITY
            }
        }
    }
    pub fn to_pivot(&self) -> Vec3 {
        match self {
            Direction::North => vec3(0.0, 0.0, -PLAYER_SIZE.z / 2.0),
            Direction::West => vec3(-PLAYER_SIZE.x / 2.0, 0.0, 0.0),
            Direction::South => vec3(0.0, 0.0, PLAYER_SIZE.z / 2.0),
            Direction::East => vec3(PLAYER_SIZE.x / 2.0, 0.0, 0.0),
            Direction::Left => vec3(0.0, 0.0, 0.0),
            Direction::Right => vec3(0.0, 0.0, 0.0),
            Direction::Around => vec3(0.0, 0.0, 0.0),
            Direction::SlideLeft => vec3(0.0, 0.0, 0.0),
            Direction::SlideRight => vec3(0.0, 0.0, 0.0),
            Direction::Wait => vec3(0.0, 0.0, 0.0),
        }
    }
}
