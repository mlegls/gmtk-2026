use std::time::Instant;
use bevy::prelude::*;
use crate::GRID_SIZE;

#[derive(Component, Clone, Default, Debug)]
pub struct Player;

#[derive(Component, Clone, Default, Debug)]
pub struct GridLocation(pub Vec2);
impl GridLocation {
    pub fn to_world_space(&self) -> Vec3 {
        let mut grid_location = vec3(self.0.x, 0.0, self.0.y);
        grid_location *= GRID_SIZE.extend(GRID_SIZE.y);
        grid_location
    }
    pub fn move_north(&mut self) {
        self.0.y -= 1.0;
    }
    pub fn move_south(&mut self) {
        self.0.y += 1.0;
    }
    pub fn move_west(&mut self) {
        self.0.x -= 1.0;
    }
    pub fn move_east(&mut self) {
        self.0.x += 1.0;
    }
}

#[derive(Component, Clone, Debug)]
pub struct Moving {
    pub direction: Direction,
    pub start: Instant,
    pub initial_rotation: Quat,
}

#[derive(Clone, Default, Debug)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}