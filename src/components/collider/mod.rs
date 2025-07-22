pub mod collider_type;

use collider_type::ColliderType;
use hex::{
    components::Trans,
    nalgebra::{Vector2, Vector3},
    parking_lot::RwLock,
    Id,
};
use std::{collections::HashSet, sync::Arc};

#[derive(Clone)]
pub struct Collider {
    pub collider_type: ColliderType,
    pub boundary: Option<f32>,
    pub layers: HashSet<Id>,
    pub ignore: HashSet<Id>,
    pub collisions: Vec<Id>,
    pub ghost: bool,
    pub log_collisions: bool,
}

impl Collider {
    pub fn new(
        collider_type: ColliderType,
        boundary: Option<f32>,
        layers: HashSet<Id>,
        ignore: HashSet<Id>,
        ghost: bool,
        log_collisions: bool,
    ) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            collider_type,
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
            ColliderType::Points(vec![
                Vector2::new(-dims1.x, -dims1.y),
                Vector2::new(-dims1.x, dims1.y),
                Vector2::new(dims1.x, dims1.y),
                Vector2::new(dims1.x, -dims1.y),
            ]),
            Some(dims.magnitude()),
            layer,
            ignore,
            ghost,
            log_collisions,
        )
    }

    fn transform_points(points: &[Vector2<f32>], transform: &Trans) -> Vec<Vector2<f32>> {
        let points: Vec<_> = points
            .iter()
            .cloned()
            .map(|p| (transform.matrix() * Vector3::new(p.x, p.y, 1.0)).xy())
            .collect();

        points
    }

    pub fn project(
        &self,
        axis: Vector2<f32>,
        points: &[Vector2<f32>],
        points2: &[Vector2<f32>],
    ) -> Option<(f32, Vector2<f32>)> {
        let mut a_min = None;
        let mut a_max = None;

        for p in points {
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

        for p in points2 {
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

        if a_min <= b_min && a_max >= b_min {
            Some((a_max - b_min, axis))
        } else if b_min <= a_min && b_max >= a_min {
            Some((b_max - a_min, -axis))
        } else {
            None
        }
    }

    pub fn intersecting(
        &self,
        transform: &Trans,
        transform2: &Trans,
        c2: &Self,
    ) -> Option<Vector2<f32>> {
        let mut min = None;

        match (&self.collider_type, &c2.collider_type) {
            (ColliderType::Points(points), ColliderType::Points(points2)) => {
                let points = Self::transform_points(&points, &transform);
                let points2 = Self::transform_points(&points2, &transform2);

                for i in 0..points.len() {
                    let p1 = points[i];
                    let p2 = points[(i + 1) % points.len()];
                    let axis = Vector2::new(p2.y - p1.y, p1.x - p2.x).normalize();
                    let (m, axis) = self.project(axis, &points, &points2)?;

                    if min.map(|(min, _)| m < min).unwrap_or(true) {
                        min = Some((m, axis));
                    }
                }
            }
            (ColliderType::Points(points), ColliderType::Circle(radius))
            | (ColliderType::Circle(radius), ColliderType::Points(points)) => {
                let (transform, transform2) = if let ColliderType::Points(_) = self.collider_type {
                    (transform, transform2)
                } else {
                    (transform2, transform)
                };
                let points = Self::transform_points(points, transform);

                for i in 0..points.len() {
                    let p1 = points[i];
                    let p2 = points[(i + 1) % points.len()];
                    let angle = (p1.x - p2.x).atan2(p1.y - p2.y);
                    let axis = Vector2::new(p2.y - p1.y, p1.x - p2.x).normalize();
                    let v2 = Vector2::new(
                        transform2.position().x + transform2.scale().x * radius * angle.cos(),
                        transform2.position().y + transform2.scale().x * radius * angle.sin(),
                    );
                    let (m, axis) = self.project(axis, &points, &[transform2.position(), v2])?;

                    if min.map(|(min, _)| m < min).unwrap_or(true) {
                        min = Some((m, axis));
                    }
                }
            }
            (ColliderType::Circle(r), ColliderType::Circle(r2)) => {
                let c1 = transform.position();
                let c2 = transform2.position();
                let v = c1 - c2;
                let c = r * transform.scale().x + r2 * transform2.scale().x;

                if v.magnitude() < c {
                    min = Some((c, v));
                }
            }
        }

        min.map(|(m, a)| a.normalize() * m)
    }
}
