use hex::{
    components::Trans,
    nalgebra::{Vector2, Vector3},
    parking_lot::RwLock,
    Id,
};
use std::{collections::HashSet, sync::Arc};

#[derive(Clone)]
pub struct Collider {
    pub points: Vec<Vector2<f32>>,
    pub boundary: f32,
    pub layers: HashSet<Id>,
    pub ignore: HashSet<Id>,
    pub collisions: Vec<Id>,
    pub ghost: bool,
    pub log_collisions: bool,
}

impl Collider {
    pub fn new(
        points: Vec<Vector2<f32>>,
        boundary: f32,
        layers: HashSet<Id>,
        ignore: HashSet<Id>,
        ghost: bool,
        log_collisions: bool,
    ) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            points,
            boundary,
            layers,
            ignore,
            collisions: Vec::new(),
            ghost,
            log_collisions,
        }))
    }

    pub fn rect(
        dims: Vector2<f32>,
        layer: HashSet<Id>,
        ignore: HashSet<Id>,
        ghost: bool,
        log_collisions: bool,
    ) -> Arc<RwLock<Self>> {
        let dims1 = dims / 2.0;

        Self::new(
            vec![
                Vector2::new(-dims1.x, -dims1.y),
                Vector2::new(-dims1.x, dims1.y),
                Vector2::new(dims1.x, dims1.y),
                Vector2::new(dims1.x, -dims1.y),
            ],
            dims.magnitude(),
            layer,
            ignore,
            ghost,
            log_collisions,
        )
    }

    pub fn oct(
        dims: Vector2<f32>,
        layers: HashSet<Id>,
        ignore: HashSet<Id>,
        ghost: bool,
        log_collisions: bool,
    ) -> Arc<RwLock<Self>> {
        let dims1 = dims / 2.0;
        let dims2 = Vector2::new(dims1.magnitude(), dims1.magnitude());

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
            dims.magnitude(),
            layers,
            ignore,
            ghost,
            log_collisions,
        )
    }

    pub fn intersecting(
        &self,
        transform: &Trans,
        transform2: &Trans,
        c2: &Self,
    ) -> Option<Vector2<f32>> {
        let points: Vec<_> = self
            .points
            .iter()
            .cloned()
            .map(|p| (transform.matrix() * Vector3::new(p.x, p.y, 1.0)).xy())
            .collect();
        let points2: Vec<_> = c2
            .points
            .iter()
            .cloned()
            .map(|p| (transform2.matrix() * Vector3::new(p.x, p.y, 1.0)).xy())
            .collect();
        let mut min = None;

        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            let axis = Vector2::new(p2.y - p1.y, p1.x - p2.x).normalize();

            let mut a_min = None;
            let mut a_max = None;

            for p in &points {
                let projected = axis.dot(p);

                if a_min.map(|a| projected < a).unwrap_or(true) {
                    a_min = Some(projected);
                }

                if a_max.map(|a| projected > a).unwrap_or(true) {
                    a_max = Some(projected);
                }
            }

            let mut b_min = None;
            let mut b_max = None;

            for p in &points2 {
                let projected = axis.dot(p);

                if b_min.map(|b| projected < b).unwrap_or(true) {
                    b_min = Some(projected);
                }

                if b_max.map(|b| projected > b).unwrap_or(true) {
                    b_max = Some(projected);
                }
            }

            let a_max = a_max?;
            let a_min = a_min?;
            let b_max = b_max?;
            let b_min = b_min?;
            let (m, axis) = if a_min <= b_min && a_max >= b_min {
                Some((a_max - b_min, axis))
            } else if b_min <= a_min && b_max >= a_min {
                Some((b_max - a_min, -axis))
            } else {
                None
            }?;

            if min.map(|(min, _)| m < min).unwrap_or(true) {
                min = Some((m, axis));
            }
        }

        min.map(|(m, a)| a.normalize() * m)
    }
}
