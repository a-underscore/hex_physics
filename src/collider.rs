use hex::{
    cid,
    components::Transform,
    ecs::{component_manager::Component, Id},
    math::Vec2,
};

#[derive(Clone)]
pub struct Collider {
    pub points: Vec<Vec2>,
    pub layers: Vec<Id>,
    pub ignore: Vec<Id>,
    pub ghost: bool,
    pub active: bool,
    pub collisions: Vec<Id>,
}

impl Collider {
    pub fn new(
        points: Vec<Vec2>,
        layers: Vec<Id>,
        ignore: Vec<Id>,
        ghost: bool,
        active: bool,
    ) -> Self {
        Self {
            points,
            layers,
            ignore,
            ghost,
            active,
            collisions: Vec::new(),
        }
    }

    pub fn rect(dims: Vec2, layer: Vec<Id>, ignore: Vec<Id>, ghost: bool, active: bool) -> Self {
        let dims = dims / 2.0;

        Self::new(
            vec![
                Vec2::new(-dims.x(), -dims.y()),
                Vec2::new(-dims.x(), dims.y()),
                Vec2::new(dims.x(), dims.y()),
                Vec2::new(dims.x(), -dims.y()),
            ],
            layer,
            ignore,
            ghost,
            active,
        )
    }

    pub fn oct(dims: Vec2, layers: Vec<Id>, ignore: Vec<Id>, ghost: bool, active: bool) -> Self {
        let dims1 = dims / 2.0;
        let dims2 = Vec2([dims1.magnitude(); 2]);

        Self::new(
            vec![
                Vec2::new(-dims1.x(), -dims1.y()),
                Vec2::new(-dims2.x(), 0.0),
                Vec2::new(-dims1.x(), dims1.y()),
                Vec2::new(0.0, dims2.y()),
                Vec2::new(dims1.x(), dims1.y()),
                Vec2::new(dims2.x(), 0.0),
                Vec2::new(dims1.x(), -dims1.y()),
                Vec2::new(0.0, -dims2.y()),
            ],
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
    ) -> Option<Vec2> {
        let a_points = self
            .points
            .iter()
            .cloned()
            .map(|p| (transform.matrix() * (p, 1.0)).0)
            .collect::<Vec<_>>();
        let b_points = b
            .points
            .iter()
            .cloned()
            .map(|p| (b_transform.matrix() * (p, 1.0)).0)
            .collect::<Vec<_>>();

        let mut min = None;

        for i in 0..a_points.len() {
            let p1 = a_points[i];
            let p2 = a_points[(i + 1) % a_points.len()];

            let axis = Vec2::new(p2.y() - p1.y(), p1.x() - p2.x()).normal();

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

        min.map(|(m, a)| a * m)
    }
}

impl Component for Collider {
    fn id() -> Id {
        cid!()
    }
}
