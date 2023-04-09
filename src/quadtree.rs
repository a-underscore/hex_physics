use hex::math::Vec2;

#[derive(Default, Clone)]
pub struct QuadTree<T> {
    pub boundary: Box2,
    pub cap: usize,
    pub points: Vec<(Vec2, Option<T>)>,
    pub nw: Option<Box<Self>>,
    pub ne: Option<Box<Self>>,
    pub se: Option<Box<Self>>,
    pub sw: Option<Box<Self>>,
}

impl<T> QuadTree<T>
where
    T: Clone,
{
    pub fn new(boundary: Box2, cap: usize) -> Self {
        Self {
            boundary,
            cap,
            points: Vec::new(),
            nw: None,
            ne: None,
            se: None,
            sw: None,
        }
    }

    pub fn insert(&mut self, point: Vec2, t: T) -> bool {
        if !self.boundary.contains(point) {
            return false;
        }

        if self.points.len() < self.cap && self.ne.is_none() {
            self.points.push((point, Some(t)));

            return true;
        }

        if self.ne.is_none() {
            self.subdivide();
        }

        self.nw
            .as_mut()
            .map(|nw| nw.insert(point, t.clone()))
            .unwrap_or_default()
            || self
                .ne
                .as_mut()
                .map(|ne| ne.insert(point, t.clone()))
                .unwrap_or_default()
            || self
                .sw
                .as_mut()
                .map(|sw| sw.insert(point, t.clone()))
                .unwrap_or_default()
            || self
                .se
                .as_mut()
                .map(|se| se.insert(point, t.clone()))
                .unwrap_or_default()
    }

    pub fn subdivide(&mut self) {
        let sub_boxes = self.boundary.subdivide();

        self.nw = Some(Box::new(QuadTree::new(sub_boxes.0, self.cap)));
        self.ne = Some(Box::new(QuadTree::new(sub_boxes.1, self.cap)));
        self.se = Some(Box::new(QuadTree::new(sub_boxes.2, self.cap)));
        self.sw = Some(Box::new(QuadTree::new(sub_boxes.3, self.cap)));
    }

    pub fn query(&mut self, range: Box2) -> Option<Vec<(Vec2, Option<T>)>> {
        let mut points = Vec::new();

        if !self.boundary.intersects(&range) {
            return None;
        }

        for v @ (point, _) in &self.points {
            if range.contains(*point) {
                points.push(v.clone());
            }
        }

        if self.ne.is_none() {
            return Some(points);
        }

        points.append(&mut self.clone().nw?.query(range.clone())?);
        points.append(&mut self.clone().ne?.query(range.clone())?);
        points.append(&mut self.clone().sw?.query(range.clone())?);
        points.append(&mut self.clone().se?.query(range)?);

        Some(points)
    }
}

#[derive(Clone, Default)]
pub struct Box2 {
    pub center: Vec2,
    pub half: f32,
}

impl Box2 {
    pub fn new(center: Vec2, half: f32) -> Self {
        Self { center, half }
    }

    pub fn contains(&self, point: Vec2) -> bool {
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

    pub fn subdivide(&self) -> (Box2, Box2, Box2, Box2) {
        let half = self.half / 2.0;

        (
            Box2::new(
                Vec2::new(self.center.x() - half, self.center.y() + half),
                half,
            ),
            Box2::new(
                Vec2::new(self.center.x() + half, self.center.y() + half),
                half,
            ),
            Box2::new(
                Vec2::new(self.center.x() + half, self.center.y() - half),
                half,
            ),
            Box2::new(
                Vec2::new(self.center.x() - half, self.center.y() - half),
                half,
            ),
        )
    }
}
