use hex::{cgmath::Vector2, cid, ecs::component_manager::Component};

#[derive(Clone)]
pub struct Physical {
    pub velocity: Vector2<f32>,
    pub active: bool,
}

impl Physical {
    pub fn new(velocity: Vector2<f32>, active: bool) -> Self {
        Self { velocity, active }
    }
}

impl From<Physical> for Vector2<f32> {
    fn from(val: Physical) -> Self {
        val.velocity
    }
}

impl Component for Physical {
    fn id() -> usize {
        cid!()
    }
}
