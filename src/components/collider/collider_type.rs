use hex::nalgebra::Vector2;

#[derive(Clone)]
pub enum ColliderType {
    Circle(f32),
    Points(Vec<Vector2<f32>>),
}
