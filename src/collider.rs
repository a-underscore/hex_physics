use hex::{
    components::Transform,
    ecs::{component_manager::Component, Id},
    math::Vec2d,
};

#[derive(Clone)]
pub struct Collider {
    pub points: Vec<Vec2d>,
    pub boundary: f32,
    pub layers: Vec<Id>,
    pub ignore: Vec<Id>,
    pub collisions: Vec<Id>,
    pub ghost: bool,
    pub active: bool,
}

impl Collider {
    pub fn new(
        points: Vec<Vec2d>,
        boundary: f32,
        layers: Vec<Id>,
        ignore: Vec<Id>,
        ghost: bool,
        active: bool,
    ) -> Self {
        Self {
            points,
            boundary,
            layers,
            ignore,
            collisions: Vec::new(),
            ghost,
            active,
        }
    }

    pub fn rect(dims: Vec2d, layer: Vec<Id>, ignore: Vec<Id>, ghost: bool, active: bool) -> Self {
        let dims1 = dims / 2.0;

        Self::new(
            vec![
                Vec2d::new(-dims1.x(), -dims1.y()),
                Vec2d::new(-dims1.x(), dims1.y()),
                Vec2d::new(dims1.x(), dims1.y()),
                Vec2d::new(dims1.x(), -dims1.y()),
            ],
            dims.magnitude(),
            layer,
            ignore,
            ghost,
            active,
        )
    }

    pub fn oct(dims: Vec2d, layers: Vec<Id>, ignore: Vec<Id>, ghost: bool, active: bool) -> Self {
        let dims1 = dims / 2.0;
        let dims2 = Vec2d([dims1.magnitude(); 2]);

        Self::new(
            vec![
                Vec2d::new(-dims1.x(), -dims1.y()),
                Vec2d::new(-dims2.x(), 0.0),
                Vec2d::new(-dims1.x(), dims1.y()),
                Vec2d::new(0.0, dims2.y()),
                Vec2d::new(dims1.x(), dims1.y()),
                Vec2d::new(dims2.x(), 0.0),
                Vec2d::new(dims1.x(), -dims1.y()),
                Vec2d::new(0.0, -dims2.y()),
            ],
            dims.magnitude(),
            layers,
            ignore,
            ghost,
            active,
        )
    }

    pub fn intersecting(
        &self,
        transform: &Transform,
        b: &Self,
        b_transform: &Transform,
    ) -> Option<Vec2d> {
        let a_points: Vec<_> = self
            .points
            .iter()
            .cloned()
            .map(|p| (transform.matrix() * (p, 1.0)).0)
            .collect();
        let b_points: Vec<_> = b
            .points
            .iter()
            .cloned()
            .map(|p| (b_transform.matrix() * (p, 1.0)).0)
            .collect();

        let mut min = None;

        for i in 0..a_points.len() {
            let p1 = a_points[i];
            let p2 = a_points[(i + 1) % a_points.len()];
            let axis = Vec2d::new(p2.y() - p1.y(), p1.x() - p2.x()).normal();

            let mut a_min = None;
            let mut a_max = None;

            for p in &a_points {
                let projected = axis.dot(*p);

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
                let projected = axis.dot(*p);

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

        min.map(|(m, a)| a.normal() * m)
    }
}

impl Component for Collider {}
