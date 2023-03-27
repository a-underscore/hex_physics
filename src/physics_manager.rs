use crate::{Collider, Physical};
use hex::{
    anyhow,
    components::Transform,
    ecs::{ev::Control, system_manager::System, Ev, World},
    glium::glutin::event::Event,
    math::Vec2,
};
use rayon::prelude::*;
use std::time::{Duration, Instant};

pub type Collision = (bool, (Option<Vec2>, Option<Vec2>));

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
        (ac, at, ap): (Collider, Transform, Option<Physical>),
        (bc, bt, bp): (Collider, Transform, Option<Physical>),
    ) -> Option<Collision> {
        if ac.layers.iter().any(|a| bc.layers.contains(a))
            && !ac.ignore.iter().any(|a| bc.layers.contains(a))
            && !bc.ignore.iter().any(|b| ac.layers.contains(b))
        {
            if let Some(min_translation) = ac.intersecting(&at, &bc, &bt) {
                return Some((
                    ac.ghost || bc.ghost,
                    (ap.map(|_| -min_translation), bp.map(|_| min_translation)),
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
    ) {
        if let Some(c) = world
            .cm
            .get_cache_mut::<Collider>(cache_collider)
            .and_then(|c| (!c.collisions.contains(&other_e)).then_some(c))
        {
            c.collisions.push(other_e);
        }

        if let Some((tr, t)) = tr.and_then(|tr| {
            (!ghost_col).then_some((tr, world.cm.get_cache_mut::<Transform>(cache_transform)?))
        }) {
            t.set_position(t.position() + tr);
        }
    }

    pub fn check_collisions(
        &mut self,
        mut entities: Vec<(usize, usize, usize)>,
        world: &mut World,
    ) {
        while let Some((ae, ac, at)) = entities.pop() {
            for ((be, bc, bt), (ghost, (atr, btr))) in entities
                .iter()
                .cloned()
                .filter_map(|(be, bc, bt)| {
                    Some((
                        (
                            world.cm.get_cache::<Collider>(ac)?.clone(),
                            world.cm.get_cache::<Transform>(at)?.clone(),
                            world.cm.get::<Physical>(ae, &world.em).cloned(),
                        ),
                        (
                            (be, bc, bt),
                            (
                                world.cm.get_cache::<Collider>(bc)?.clone(),
                                world.cm.get_cache::<Transform>(bt)?.clone(),
                                world.cm.get::<Physical>(be, &world.em).cloned(),
                            ),
                        ),
                    ))
                })
                .collect::<Vec<_>>()
                .into_par_iter()
                .filter_map(|(a, ((be, bc, bt), b))| Some(((be, bc, bt), Self::detect(a, b)?)))
                .collect::<Vec<_>>()
            {
                Self::resolve(ghost, ae, bc, bt, btr, world);
                Self::resolve(ghost, be, ac, at, atr, world);
            }
        }
    }
}

impl<'a> System<'a> for PhysicsManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            let now = Instant::now();
            let delta = now.duration_since(self.frame);

            self.frame = now;

            let entities: Vec<_> = world
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

                                    col.active.then_some(c)
                                })
                            })?,
                        world
                            .cm
                            .get_cache_id::<Transform>(e, &world.em)
                            .and_then(|t| {
                                world.cm.get_cache::<Transform>(t)?.active.then_some(t)
                            })?,
                    ))
                })
                .collect();

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

                            self.check_collisions(entities.clone(), world);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
