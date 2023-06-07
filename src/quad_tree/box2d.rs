use hex::math::Vec2d;

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
