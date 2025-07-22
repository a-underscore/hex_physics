use hex::Vector2;

#[derive(Clone)]
pub enum ColliderType {
    Circle(f32),
    Points((f32, Vec<Vector2<f32>>)),
}
