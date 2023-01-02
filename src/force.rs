use hex::{cgmath::Vector2, cid, ecs::component_manager::Component};

pub struct Kilos(f32);
pub struct Mps(f32);

pub struct Force {
    pub weight: Kilos,
    pub velocity: Vector2<Mps>,
}

impl Component for Force {
    fn id() -> usize {
        cid!()
    }
}
