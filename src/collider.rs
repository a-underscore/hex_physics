use hex::{
    cgmath::{InnerSpace, Vector2},
    cid,
    components::Transform,
    ecs::component_manager::Component,
};

#[derive(Clone)]
pub struct Collider {
    pub points: Vec<Vector2<f32>>,
    pub active: bool,
}

impl Collider {
    pub fn new(points: Vec<Vector2<f32>>, active: bool) -> Self {
        Self { points, active }
    }

    pub fn rect(dims: Vector2<f32>, active: bool) -> Self {
        let dims = dims / 2.0;

        Self::new(
            vec![
                Vector2::new(-dims.x, -dims.y),
                Vector2::new(-dims.x, dims.y),
                Vector2::new(dims.x, dims.y),
                Vector2::new(dims.x, -dims.y),
            ],
            active,
        )
    }

    pub fn oct(dims: Vector2<f32>, active: bool) -> Self {
        let dims1 = dims / 2.0;
        let dims2 = Vector2::from([dims1.magnitude(); 2]);

        Self::new(
            vec![
                Vector2::new(-dims1.x, -dims1.y),
                Vector2::new(-dims2.x, 0.0),
                Vector2::new(-dims1.x, dims1.y),
                Vector2::new(0.0, dims2.y),
                Vector2::new(dims1.x, dims1.y),
                Vector2::new(dims2.x, 0.0),
                Vector2::new(dims1.x, -dims1.y),
                Vector2::new(0.0, -dims2.y),
            ],
            active,
        )
    }

    pub fn intersecting(
        &self,
        transform: &Transform,
        b: &Self,
        b_transform: &Transform,
    ) -> Option<f32> {
        let a_points = self
            .points
            .iter()
            .cloned()
            .map(|p| (transform.matrix() * p.extend(1.0)).truncate())
            .collect::<Vec<_>>();
        let b_points = b
            .points
            .iter()
            .cloned()
            .map(|p| (b_transform.matrix() * p.extend(1.0)).truncate())
            .collect::<Vec<_>>();

        let mut min = None;

        for i in 0..a_points.len() {
            let p1 = a_points[i];
            let p2 = a_points[(i + 1) % a_points.len()];

            let normal = Vector2::new(p2.y - p1.y, p1.x - p2.x);

            let mut a_min = None;
            let mut a_max = None;

            for p in &a_points {
                let projected = normal.x * p.x + normal.y * p.y;

                if a_min.map(|a| projected < a).unwrap_or(true) {
                    a_min = Some(projected);
                }

                if a_max.map(|a| projected > a).unwrap_or(true) {
                    a_max = Some(projected);
                }
            }

            let mut b_min = None;
            let mut b_max = None;

            for p in &b_points {
                let projected = normal.dot(*p);

                if b_min.map(|b| projected < b).unwrap_or(true) {
                    b_min = Some(projected);
                }

                if b_max.map(|b| projected > b).unwrap_or(true) {
                    b_max = Some(projected);
                }
            }

            if let (Some(a_min), Some(a_max), Some(b_min), Some(b_max)) =
                (a_min, a_max, b_min, b_max)
            {
                if !(a_max < b_min || b_max < a_min) {
                    let m = (a_max - b_min).min(b_max - b_min);

                    if min.map(|min| m < min).unwrap_or(true) {
                        min = Some(m);
                    }

                    continue;
                }
            }

            return None;
        }

        min
    }
}

impl Component for Collider {
    fn id() -> usize {
        cid!()
    }
}
