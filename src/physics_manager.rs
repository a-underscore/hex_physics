use crate::{Collider, Physical};
use hex::{
    anyhow,
    components::Transform,
    ecs::{ev::Control, system_manager::System, ComponentManager, Context, EntityManager, Ev, Id},
    glium::glutin::event::Event,
    math::Vec2d,
};
use rayon::prelude::*;
use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

pub type Colliders = Vec<(Id, (Id, Collider), Id, Option<Physical>)>;

pub struct PhysicsManager {
    pub rate: u32,
    pub step_amount: u32,
    pub max_delta: Option<Duration>,
    frame: Instant,
    count: u32,
}

impl PhysicsManager {
    pub fn new(rate: u32, step_amount: u32, max_delta: Option<Duration>) -> Self {
        Self {
            rate,
            step_amount,
            max_delta,
            frame: Instant::now(),
            count: 0,
        }
    }

    pub fn detect(
        (ac, at, ap): (&Collider, &Transform, Option<&Physical>),
        (bc, bt, bp): (&Collider, &Transform, Option<&Physical>),
    ) -> Option<(Option<Vec2d>, Option<Vec2d>)> {
        if ac.layers.iter().any(|a| bc.layers.contains(a))
            && !(ac.ignore.iter().any(|a| bc.layers.contains(a))
                || bc.ignore.iter().any(|b| ac.layers.contains(b)))
            && (at.position() - bt.position()).magnitude() <= ac.boundary + bc.boundary
        {
            if let Some(min_translation) = ac.intersecting(at, bc, bt) {
                return Some(
                    (!(ac.ghost || bc.ghost))
                        .then(|| {
                            (
                                ap.as_ref().is_some().then_some(-min_translation),
                                bp.as_ref().is_some().then_some(min_translation),
                            )
                        })
                        .unwrap_or_default(),
                );
            }
        }

        None
    }

    pub fn resolve(
        other_e: Id,
        cache_collider: Id,
        cache_transform: Id,
        tr: Option<Vec2d>,
        cm: &mut ComponentManager,
    ) {
        if let Some(collider) = cm.get_cache_mut::<Collider>(cache_collider) {
            if !collider.collisions.contains(&other_e) {
                collider.collisions.push(other_e);
            }

            if let Some((tr, t)) =
                tr.and_then(|tr| Some((tr, cm.get_cache_mut::<Transform>(cache_transform)?)))
            {
                t.set_position(t.position() + tr);
            }
        }
    }

    pub fn check_collisions(&self, (em, cm): (&EntityManager, &mut ComponentManager)) {
        let entities: Vec<_> = em
            .entities
            .keys()
            .cloned()
            .filter_map(|e| {
                let e = (
                    e,
                    cm.get_id::<Collider>(e, em).and_then(|c| {
                        cm.get_cache::<Collider>(c)
                            .and_then(|col| col.active.then_some((c, col)))
                    })?,
                    cm.get_id::<Transform>(e, em).and_then(|t| {
                        cm.get_cache::<Transform>(t)
                            .and_then(|transform| transform.active.then_some((t, transform)))
                    })?,
                    cm.get::<Physical>(e, em)
                        .and_then(|p| p.active.then_some(p)),
                );

                Some(e)
            })
            .collect();
        let res: Vec<_> = {
            let checked = RwLock::new(Vec::new());

            entities
                .par_iter()
                .filter_map(|(ae, (ac, a_col), (at, a_transform), a_physical)| {
                    let res: Vec<_> = entities
                        .iter()
                        .filter_map(|(be, (bc, b_col), (bt, b_transform), b_physical)| {
                            let res = {
                                let checked = checked.read().ok()?;

                                !checked.contains(&(ae, *be)) && !checked.contains(&(be, *ae))
                            };
                            let res = if res {
                                Some((
                                    (*ae, *ac, *at),
                                    (*be, *bc, *bt),
                                    Self::detect(
                                        (a_col, a_transform, *a_physical),
                                        (b_col, b_transform, *b_physical),
                                    )?,
                                ))
                            } else {
                                None
                            };

                            checked.write().ok()?.push((ae, *be));

                            res
                        })
                        .collect();

                    Some(res)
                })
                .flatten()
                .collect()
        };

        for ((ae, ac, at), (be, bc, bt), (atr, btr)) in res {
            Self::resolve(ae, bc, bt, btr, cm);
            Self::resolve(be, ac, at, atr, cm);
        }
    }

    pub fn update_positions(
        &self,
        step_amount: Option<u32>,
        delta: Duration,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) {
        for e in em.entities.keys().cloned() {
            if let Some((force, t)) = cm.get::<Physical>(e, em).cloned().and_then(|p| {
                Some((
                    p.active.then_some(p.force)?,
                    cm.get_mut::<Transform>(e, em)
                        .and_then(|t| t.active.then_some(t))?,
                ))
            }) {
                if let Some(step_amount) = step_amount {
                    t.set_position(
                        t.position() + (force * delta.as_secs_f32()) / step_amount as f32,
                    );

                    self.check_collisions((em, cm));
                } else {
                    t.set_position(t.position() + force * delta.as_secs_f32());
                }
            }
        }
    }

    pub fn clear_collisions(&self, (em, cm): (&mut EntityManager, &mut ComponentManager)) {
        for e in em.entities.keys().cloned() {
            if let Some(col) = cm
                .get_mut::<Collider>(e, em)
                .and_then(|col| col.active.then_some(col))
            {
                col.collisions.clear()
            }
        }
    }

    pub fn update_velocities(
        &self,
        delta: Duration,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) {
        for e in em.entities.keys().cloned() {
            if let Some((t, p)) = cm.get::<Transform>(e, em).cloned().and_then(|t| {
                Some((
                    t.active.then_some(t)?,
                    cm.get_mut::<Physical>(e, em)
                        .and_then(|p| p.active.then_some(p))?,
                ))
            }) {
                p.set_velocity((t.position() - p.last_position()) / delta.as_secs_f32());
                p.set_last_position(t.position());
            }
        }
    }
}

impl<'a> System<'a> for PhysicsManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        _: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            let now = Instant::now();
            let delta = {
                let delta = now.duration_since(self.frame);

                if let Some(md) = self.max_delta {
                    delta.min(md)
                } else {
                    delta
                }
            };

            self.frame = now;

            self.clear_collisions((em, cm));

            if self.count >= self.rate {
                self.count = 0;

                for _ in 0..self.step_amount {
                    self.update_positions(Some(self.step_amount), delta, (em, cm));
                }
            } else {
                self.update_positions(None, delta, (em, cm));
            }

            self.update_velocities(delta, (em, cm));

            self.count += 1;
        }

        Ok(())
    }
}
