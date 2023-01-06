use hex::{cgmath::Vector2, cid, ecs::component_manager::Component};

pub struct Force {
    pub weight: f32,
    pub velocity: Vector2<f32>,
}

impl Force {
    pub fn new(weight: f32, velocity: Vector2<f32>) -> Self {
        Self { weight, velocity }
    }
}

impl Component for Force {
    fn id() -> usize {
        cid!()
    }
}
