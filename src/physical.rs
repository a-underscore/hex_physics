use hex::{
    ecs::{component_manager::Component, Id},
    id,
    math::Vec2d,
};

#[derive(Clone)]
pub struct Physical {
    pub force: Vec2d,
    velocity: Option<Vec2d>,
    last_position: Option<Vec2d>,
    pub active: bool,
}

impl Physical {
    pub fn new(force: Vec2d, active: bool) -> Self {
        Self {
            force,
            velocity: None,
            last_position: None,
            active,
        }
    }

    pub fn last_position(&self) -> Option<Vec2d> {
        self.last_position
    }

    pub fn set_last_position(&mut self, lp: Vec2d) {
        self.last_position = Some(lp);
    }

    pub fn velocity(&self) -> Vec2d {
        self.velocity.unwrap_or_default()
    }

    pub fn set_velocity(&mut self, vel: Vec2d) {
        self.velocity = Some(vel);
    }
}

impl Component for Physical {
    fn id() -> Id {
        id!()
    }
}
