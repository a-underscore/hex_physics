use hex::{
    cgmath::{InnerSpace, Vector2},
    cid,
    components::Transform,
    ecs::component_manager::Component,
};

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

    // Adapted from https://github.com/winstxnhdw/2d-separating-axis-theorem

    fn normalized_proj_axis(current: Vector2<f32>, next: Vector2<f32>) -> Vector2<f32> {
        Vector2::new(-(next.y - current.y), next.x - current.x).normalize()
    }

    fn projs(
        a_points: &[Vector2<f32>],
        b_points: &[Vector2<f32>],
        axis_normalized: Vector2<f32>,
    ) -> (Vec<f32>, Vec<f32>) {
        let mut a_projs = Vec::new();
        let mut b_projs = Vec::new();

        for (a, b) in a_points.iter().zip(b_points.iter()) {
            let a_proj = axis_normalized.dot(*a);
            let b_proj = axis_normalized.dot(*b);

            a_projs.push(a_proj);
            b_projs.push(b_proj);
        }

        (a_projs, b_projs)
    }

    fn overlapping(projs_a: Vec<f32>, projs_b: Vec<f32>) -> bool {
        let max_a = projs_a.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
        let min_a = projs_a.iter().min_by(|a, b| a.partial_cmp(b).unwrap());
        let max_b = projs_b.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
        let min_b = projs_b.iter().min_by(|a, b| a.partial_cmp(b).unwrap());

        !(max_a < min_b || max_b < min_a)
    }

    pub fn intersecting(&self, transform: &Transform, b: &Self, b_transform: &Transform) -> bool {
        let a_bounds = self
            .points
            .iter()
            .cloned()
            .map(|p| (transform.matrix() * p.extend(1.0)).truncate())
            .collect::<Vec<_>>();
        let b_bounds = b
            .points
            .iter()
            .cloned()
            .map(|p| (b_transform.matrix() * p.extend(1.0)).truncate())
            .collect::<Vec<_>>();

        for i in 0..a_bounds.len() {
            let current = a_bounds[i];
            let next = a_bounds[(i + 1) % a_bounds.len()];
            let axis = Self::normalized_proj_axis(current, next);
            let (a_projs, b_projs) = Self::projs(&a_bounds, &b_bounds, axis);

            if !Self::overlapping(a_projs, b_projs) {
                return false;
            }
        }

        for i in 0..b_bounds.len() {
            let current = b_bounds[i];
            let next = b_bounds[(i + 1) % a_bounds.len()];
            let axis = Self::normalized_proj_axis(current, next);
            let (a_projs, b_projs) = Self::projs(&a_bounds, &b_bounds, axis);

            if !Self::overlapping(a_projs, b_projs) {
                return false;
            }
        }

        true
    }

    // End adapted code
}

impl Component for Polygon {
    fn id() -> usize {
        cid!()
    }
}
