use hex::{
    ecs::{component_manager::Component, Id},
    id,
    math::Vec2,
};

#[derive(Clone)]
pub struct Physical {
    pub force: Vec2,
    velocity: Option<Vec2>,
    last_position: Option<Vec2>,
    pub active: bool,
}

impl Physical {
    pub fn new(force: Vec2, active: bool) -> Self {
        Self {
            force,
            velocity: None,
            last_position: None,
            active,
        }
    }

    pub fn last_position(&self) -> Option<Vec2> {
        self.last_position
    }

    pub fn set_last_position(&mut self, lp: Vec2) {
        self.last_position = Some(lp);
    }

    pub fn velocity(&self) -> Option<Vec2> {
        self.velocity
    }

    pub fn set_velocity(&mut self, vel: Vec2) {
        self.velocity = Some(vel);
    }
}

impl Component for Physical {
    fn id() -> Id {
        id!()
    }
}
