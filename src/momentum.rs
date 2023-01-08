use hex::{cgmath::Vector2, cid, ecs::component_manager::Component};

#[derive(Clone)]
pub struct Momentum {
    pub mass: f32,
    pub velocity: Vector2<f32>,
    pub applied: Vec<usize>,
}

impl Momentum {
    pub fn new(mass: f32, velocity: Vector2<f32>) -> Self {
        Self {
            mass,
            velocity,
            applied: Vec::new(),
        }
    }

    pub fn apply_force(&mut self, other: usize) {
        self.applied.push(other);
    }
}

impl Into<Vector2<f32>> for Momentum {
    fn into(self) -> Vector2<f32> {
        self.velocity * self.mass
    }
}

impl Component for Momentum {
    fn id() -> usize {
        cid!()
    }
}
