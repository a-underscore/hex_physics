use hex::{cid, ecs::component_manager::Component, math::Vec2};

#[derive(Clone)]
pub struct Physical {
    pub velocity: Vec2,
    pub active: bool,
}

impl Physical {
    pub fn new(velocity: Vec2, active: bool) -> Self {
        Self { velocity, active }
    }
}

impl From<Physical> for Vec2 {
    fn from(val: Physical) -> Self {
        val.velocity
    }
}

impl Component for Physical {
    fn id() -> usize {
        cid!()
    }
}
