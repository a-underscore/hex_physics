use hex::{
    cgmath::{InnerSpace, Vector2},
    cid,
    components::Transform,
    ecs::{
        component_manager::{Component, ComponentManager},
        entity_manager::EntityManager,
    },
};
use std::{cell::RefCell, f32::INFINITY, rc::Rc};

pub trait Callback<'a>:
    FnMut(usize, usize, &mut EntityManager, &mut ComponentManager) + 'a
{
}

pub struct Polygon<'a> {
    pub points: Vec<Vector2<f32>>,
    pub callback: Rc<RefCell<dyn Callback<'a>>>,
}

impl<'a> Polygon<'a> {
    pub fn new<C>(points: Vec<Vector2<f32>>, callback: C) -> Self
    where
        C: Callback<'a>,
    {
        Self {
            points,
            callback: Rc::new(RefCell::new(callback)),
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
                let proj = axis.dot(*p);

                if proj < min_proj {
                    min_proj = proj;
                }

                if proj > max_proj {
                    max_proj = proj;
                }
            }

            for p in &other_points {
                let proj = axis.dot(*p);

                if proj < other_min_proj {
                    other_min_proj = proj;
                }

                if proj > other_max_proj {
                    other_max_proj = proj;
                }
            }

            if max_proj < other_min_proj || min_proj > other_max_proj {
                return true;
            }
        }

        false
    }
}

impl<'a> Component for Polygon<'a> {
    fn id() -> usize {
        cid!()
    }
}
