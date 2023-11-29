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
            let min_translation = ac.intersecting(at, bc, bt)?;

            Some(
                (!(ac.ghost || bc.ghost))
                    .then_some({
                        (
                            ap.is_some().then_some(-min_translation),
                            bp.is_some().then_some(min_translation),
                        )
                    })
                    .unwrap_or_default(),
            )
        } else {
            None
        }
    }

    pub fn resolve(e: Id, other_e: Id, tr: Option<Vec2d>, cm: &mut ComponentManager) {
        if let Some(collider) = cm.get_mut::<Collider>(e) {
            if !collider.collisions.contains(&other_e) {
                collider.collisions.push(other_e);
            }

            if let Some((tr, t)) = tr.and_then(|tr| Some((tr, cm.get_mut::<Transform>(e)?))) {
                t.set_position(t.position() + tr);
            }
        }
    }

    pub fn check_collisions(&self, (em, cm): (&EntityManager, &mut ComponentManager)) {
        let entities: Vec<_> = em
            .entities()
            .filter_map(|e| {
                let collider = cm
                    .get::<Collider>(e)
                    .and_then(|c| c.active.then_some(c.clone()))?;
                let transform = cm
                    .get::<Transform>(e)
                    .and_then(|t| t.active.then_some(t.clone()))?;
                let physical = cm
                    .get::<Physical>(e)
                    .and_then(|p| p.active.then_some(p))
                    .and_then(|physical| {
                        Some((
                            (collider.convex_hull(&transform, physical)?),
                            transform.clone(),
                            Some(physical),
                        ))
                    })
                    .unwrap_or((collider, transform, None));

                Some((e, physical))
            })
            .collect();
        let checked = RwLock::new(Vec::new());
        let col: Vec<_> = entities
            .par_iter()
            .map(|(ae, (a_col, a_transform, a_physical))| {
                let res: Vec<_> = entities
                    .iter()
                    .filter_map(|(be, (b_col, b_transform, b_physical))| {
                        let res = {
                            let res = {
                                let checked = checked.read().ok()?;

                                !checked.contains(&(*ae, *be)) && !checked.contains(&(*be, *ae))
                            };

                            if res {
                                checked.write().ok()?.push((*ae, *be));

                                true
                            } else {
                                false
                            }
                        };

                        if res {
                            Self::detect(
                                (a_col, a_transform, *a_physical),
                                (b_col, b_transform, *b_physical),
                            )
                            .map(|tr| (*ae, *be, tr))
                        } else {
                            None
                        }
                    })
                    .collect();

                res
            })
            .flatten()
            .collect();

        for (ae, be, (atr, btr)) in col {
            Self::resolve(ae, be, atr, cm);
            Self::resolve(be, ae, btr, cm);
        }
    }

    pub fn update_positions(
        &self,
        step_amount: Option<u32>,
        delta: Duration,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) {
        for e in em.entities() {
            if let Some((force, t)) = cm.get::<Physical>(e).cloned().and_then(|p| {
                Some((
                    p.active.then_some(p.force)?,
                    cm.get_mut::<Transform>(e)
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
        for e in em.entities() {
            if let Some(col) = cm
                .get_mut::<Collider>(e)
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
        for e in em.entities() {
            if let Some((t, p)) = cm.get::<Transform>(e).cloned().and_then(|t| {
                Some((
                    t.active.then_some(t)?,
                    cm.get_mut::<Physical>(e)
                        .and_then(|p| p.active.then_some(p))?,
                ))
            }) {
                p.set_velocity(
                    (t.position() - p.last_position().unwrap_or_default()) / delta.as_secs_f32(),
                );
                p.set_last_position(t.position());
            }
        }
    }
}

impl System for PhysicsManager {
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
