use hex::{
    ecs::{component_manager::Component, Id},
    id,
    math::Vec2,
};

#[derive(Clone)]
pub struct Physical {
    pub velocity: Vec2,
    pub active: bool,
    pub last_position: Option<Vec2>,
}

impl Physical {
    pub fn new(velocity: Vec2, active: bool) -> Self {
        Self {
            velocity,
            active,
            last_position: None,
        }
    }
}

impl Component for Physical {
    fn id() -> Id {
        id!()
    }
}
