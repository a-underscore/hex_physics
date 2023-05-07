use hex::{ecs::Id, math::Vec2d};
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct QuadTree<T> {
    pub boundary: Box2d,
    pub cap: usize,
    pub points: Vec<((Vec2d, Id), Arc<T>)>,
    pub sub: Option<(Box<Self>, Box<Self>, Box<Self>, Box<Self>)>,
}

impl<T> QuadTree<T> {
    pub fn new(boundary: Box2d, cap: usize) -> Self {
        Self {
            boundary,
            cap,
            points: Vec::new(),
            sub: None,
        }
    }

    pub fn insert(&mut self, v @ (p, _): (Vec2d, Id), t: Arc<T>) -> bool {
        if !self.boundary.contains(p) {
            return false;
        }

        if self.sub.is_none() {
            if self.points.len() < self.cap {
                self.points.push((v, t));

                return true;
            }

            self.subdivide();
        }

        self.sub
            .as_mut()
            .map(|(nw, ne, sw, se)| {
                nw.insert(v, t.clone())
                    || ne.insert(v, t.clone())
                    || sw.insert(v, t.clone())
                    || se.insert(v, t.clone())
            })
            .unwrap_or_default()
    }

    pub fn subdivide(&mut self) {
        let sub_boxes = self.boundary.subdivide();

        self.sub = Some((
            Box::new(Self::new(sub_boxes.0, self.cap)),
            Box::new(Self::new(sub_boxes.1, self.cap)),
            Box::new(Self::new(sub_boxes.2, self.cap)),
            Box::new(Self::new(sub_boxes.3, self.cap)),
        ));
    }

    pub fn query(&self, range: Box2d) -> Vec<((Vec2d, Id), Arc<T>)> {
        let mut points = Vec::new();

        if !self.boundary.intersects(&range) {
            return points;
        }

        for v @ ((p, _), _) in &self.points {
            if range.contains(*p) {
                points.push(v.clone());
            }
        }

        if self.sub.is_none() {
            return points;
        }

        self.sub
            .as_ref()
            .map(|(nw, ne, sw, se)| {
                points.append(&mut nw.query(range.clone()));
                points.append(&mut ne.query(range.clone()));
                points.append(&mut sw.query(range.clone()));
                points.append(&mut se.query(range.clone()));
            })
            .unwrap_or_default();

        points
    }
}

#[derive(Clone, Default)]
pub struct Box2d {
    pub center: Vec2d,
    pub half: f32,
}

impl Box2d {
    pub fn new(center: Vec2d, half: f32) -> Self {
        Self { center, half }
    }

    pub fn contains(&self, point: Vec2d) -> bool {
        point.x() >= self.center.x() - self.half
            && point.x() <= self.center.x() + self.half
            && point.y() >= self.center.y() - self.half
            && point.y() <= self.center.y() + self.half
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.center.x() - self.half <= other.center.x() + other.half
            && self.center.x() + self.half >= other.center.x() - other.half
            && self.center.y() - self.half <= other.center.y() + other.half
            && self.center.y() + self.half >= other.center.y() - other.half
    }

    pub fn subdivide(&self) -> (Box2d, Box2d, Box2d, Box2d) {
        let half = self.half / 2.0;

        (
            Box2d::new(
                Vec2d::new(self.center.x() - half, self.center.y() + half),
                half,
            ),
            Box2d::new(
                Vec2d::new(self.center.x() + half, self.center.y() + half),
                half,
            ),
            Box2d::new(
                Vec2d::new(self.center.x() + half, self.center.y() - half),
                half,
            ),
            Box2d::new(
                Vec2d::new(self.center.x() - half, self.center.y() - half),
                half,
            ),
        )
    }
}
