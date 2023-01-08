use hex::{cgmath::Vector2, cid, ecs::component_manager::Component};

#[derive(Clone)]
pub struct Momentum {
    pub mass: f32,
    pub velocity: Vector2<f32>,
    pub applied: Vec<usize>,
    pub active: bool,
}

impl Momentum {
    pub fn new(mass: f32, velocity: Vector2<f32>, active: bool) -> Self {
        Self {
            mass,
            velocity,
            applied: Vec::new(),
            active,
        }
    }

    pub fn apply(&mut self, other: usize) {
        self.applied.push(other);
    }
}

impl From<Momentum> for Vector2<f32> {
    fn from(val: Momentum) -> Self {
        val.velocity * val.mass
    }
}

impl Component for Momentum {
    fn id() -> usize {
        cid!()
    }
}
