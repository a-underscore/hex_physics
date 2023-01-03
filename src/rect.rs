use hex::{
    cgmath::{InnerSpace, Vector2},
    cid,
    components::Transform,
    ecs::component_manager::Component,
};
use std::f32::INFINITY;

pub struct Polygon {
    pub points: Vec<Vector2<f32>>,
    pub collisions: Vec<usize>,
}

impl Polygon {
    pub fn new(points: Vec<Vector2<f32>>) -> Self {
        Self {
            points,
            collisions: Vec::new(),
        }
    }

    pub fn rect(dims: Vector2<f32>) -> Self {
        let dims = dims / 2.0;

        Self::new(vec![
            Vector2::new(-dims.x, -dims.y),
            Vector2::new(-dims.x, dims.y),
            Vector2::new(dims.x, -dims.y),
            Vector2::new(dims.x, dims.y),
        ])
    }

    fn proj(&self, axis: Vector2<f32>, p: Vector2<f32>, min_proj: &mut f32, max_proj: &mut f32) {
        let proj = p.dot(axis);

        if proj < *min_proj {
            *min_proj = proj;
        }

        if proj > *max_proj {
            *max_proj = proj;
        }
    }

    pub fn intersecting(
        &self,
        transform: &Transform,
        other: &Self,
        other_transform: &Transform,
    ) -> bool {
        let points = self
            .points
            .iter()
            .cloned()
            .map(|p| (transform.matrix() * p.extend(1.0)).truncate())
            .collect::<Vec<_>>();
        let other_points = other
            .points
            .iter()
            .cloned()
            .map(|o| (other_transform.matrix() * o.extend(1.0)).truncate())
            .collect::<Vec<_>>();

        for i in 0..points.len() {
            let current = points[i];
            let next = points[(i + 1) % points.len()];
            let edge = next - current;
            let axis = Vector2::new(-edge.y, edge.x);

            let mut max_proj = -INFINITY;
            let mut min_proj = INFINITY;
            let mut other_max_proj = -INFINITY;
            let mut other_min_proj = INFINITY;

            for p in &points {
                self.proj(axis, *p, &mut min_proj, &mut max_proj);
            }

            for p in &other_points {
                self.proj(axis, *p, &mut other_min_proj, &mut other_max_proj);
            }

            if max_proj < other_min_proj || min_proj > other_max_proj {
                return true;
            }
        }

        false
    }
}

impl Component for Polygon {
    fn id() -> usize {
        cid!()
    }
}
