use crate::{Collider, Physical};
use hex::{
    anyhow,
    components::Transform,
    ecs::{ev::Control, system_manager::System, Ev, Scene, World},
    glium::glutin::event::Event,
    math::Vec2,
};
use std::time::{Duration, Instant};

pub type Collision = (bool, (Option<Vec2>, Option<Vec2>));
pub type Colliders = (
    usize,
    (usize, Collider),
    (usize, Transform),
    Option<Physical>,
);

pub struct PhysicsManager {
    pub step_amount: usize,
    pub max_delta: Duration,
    frame: Instant,
}

impl PhysicsManager {
    pub fn new(step_amount: usize, max_delta: Duration) -> Self {
        Self {
            step_amount,
            max_delta,
            frame: Instant::now(),
        }
    }

    pub fn detect(
        (ac, at, ap): (&Collider, &Transform, &Option<Physical>),
        (bc, bt, bp): (&Collider, &Transform, &Option<Physical>),
    ) -> Option<Collision> {
        if ac.layers.iter().any(|a| bc.layers.contains(a))
            && !ac.ignore.iter().any(|a| bc.layers.contains(a))
            && !bc.ignore.iter().any(|b| ac.layers.contains(b))
        {
            if let Some(min_translation) = ac.intersecting(at, bc, bt) {
                return Some((
                    ac.ghost || bc.ghost,
                    (
                        ap.clone().map(|_| -min_translation),
                        bp.clone().map(|_| min_translation),
                    ),
                ));
            }
        }

        None
    }

    pub fn resolve(
        ghost_col: bool,
        other_e: usize,
        cache_collider: usize,
        cache_transform: usize,
        tr: Option<Vec2>,
        world: &mut World,
    ) -> Option<()> {
        world
            .cm
            .get_cache_mut::<Collider>(cache_collider)?
            .collisions
            .push(other_e);

        let (tr, t) = tr.and_then(|tr| {
            (!ghost_col).then_some((tr, world.cm.get_cache_mut::<Transform>(cache_transform)?))
        })?;

        t.set_position(t.position() + tr);

        Some(())
    }

    pub fn check_collisions(&mut self, world: &mut World) {
        let mut entities: Vec<_> = world
            .em
            .entities
            .keys()
            .cloned()
            .filter_map(|e| {
                Some((
                    e,
                    world
                        .cm
                        .get_cache_id::<Collider>(e, &world.em)
                        .and_then(|c| {
                            world.cm.get_cache_mut::<Collider>(c).and_then(|col| {
                                col.collisions.clear();

                                col.active.then(|| (c, col.clone()))
                            })
                        })?,
                    world
                        .cm
                        .get_cache_id::<Transform>(e, &world.em)
                        .and_then(|t| {
                            world.cm.get_cache::<Transform>(t).and_then(|transform| {
                                transform.active.then(|| (t, transform.clone()))
                            })
                        })?,
                    world.cm.get::<Physical>(e, &world.em).cloned(),
                ))
            })
            .collect();

        while let Some((ae, (ac, a_col), (at, a_transform), a_physical)) = entities.pop() {
            for ((be, bc, bt), (ghost, (atr, btr))) in
                entities
                    .iter()
                    .filter_map(|(be, (bc, b_col), (bt, b_transform), b_physical)| {
                        Some((
                            (be, bc, bt),
                            Self::detect(
                                (&a_col, &a_transform, &a_physical),
                                (b_col, b_transform, b_physical),
                            )?,
                        ))
                    })
            {
                Self::resolve(ghost, ae, *bc, *bt, btr, world);
                Self::resolve(ghost, *be, ac, at, atr, world);
            }
        }
    }
}

impl<'a> System<'a> for PhysicsManager {
    fn update(&mut self, ev: &mut Ev, _: &mut Scene, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            let now = Instant::now();
            let delta = now.duration_since(self.frame);

            self.frame = now;

            for _ in 0..self.step_amount {
                for e in world.em.entities.clone().into_keys() {
                    if let Some(velocity) = world
                        .cm
                        .get_mut::<Physical>(e, &world.em)
                        .and_then(|p| p.active.then_some(p.velocity))
                    {
                        if let Some(t) = world.cm.get_mut::<Transform>(e, &world.em) {
                            t.set_position(
                                t.position()
                                    + velocity / self.step_amount as f32
                                        * delta.min(self.max_delta).as_secs_f32(),
                            );

                            self.check_collisions(world);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
