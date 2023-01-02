use hex::cgmath::Vector2;

pub struct Kilos(f32);
pub struct Mps(f32);

pub struct Force {
    pub weight: Kilos,
    pub velocity: Vector2<Mps>,
}
