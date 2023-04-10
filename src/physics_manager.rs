use crate::{Box2, Collider, Physical, QuadTree};
use hex::{
    anyhow,
    components::Transform,
    ecs::{ev::Control, system_manager::System, ComponentManager, EntityManager, Ev, Id, Scene},
    glium::glutin::event::Event,
    math::Vec2,
};
use rayon::prelude::*;
use std::{
    sync::{Mutex, RwLock},
    time::{Duration, Instant},
};

pub type Collision = (bool, (Option<Vec2>, Option<Vec2>));
pub type Colliders = Vec<(Id, (Id, Collider), Id, Option<Physical>)>;

pub struct PhysicsManager {
    pub step_amount: usize,
    pub max_delta: Duration,
    pub bounds: (Box2, usize),
    frame: Instant,
}

impl PhysicsManager {
    pub fn new(step_amount: usize, max_delta: Duration, bounds: (Box2, usize)) -> Self {
        Self {
            step_amount,
            max_delta,
            bounds,
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
                        ap.as_ref().map(|_| -min_translation),
                        bp.as_ref().map(|_| min_translation),
                    ),
                ));
            }
        }

        None
    }

    pub fn resolve(
        ghost_col: bool,
        other_e: Id,
        cache_collider: Id,
        cache_transform: Id,
        tr: Option<Vec2>,
        cm: &mut ComponentManager,
    ) {
        if let Some(collider) = cm
            .get_cache_mut::<Collider>(cache_collider)
            .and_then(|c| (!c.collisions.contains(&other_e)).then_some(c))
        {
            collider.collisions.push(other_e);
        }

        if let Some((tr, t)) = tr.and_then(|tr| {
            (!ghost_col).then_some((tr, cm.get_cache_mut::<Transform>(cache_transform)?))
        }) {
            t.set_position(t.position() + tr);
        }
    }

    pub fn check_collisions(&mut self, (em, cm): (&EntityManager, &mut ComponentManager)) {
        let (boundary, cap) = self.bounds.clone();
        let tree = Mutex::new(QuadTree::new(boundary, cap));
        let entities: Vec<_> = em
            .entities
            .keys()
            .cloned()
            .filter_map(|e| {
                Some((
                    e,
                    cm.get_cache_id::<Collider>(e, em).and_then(|c| {
                        cm.get_cache_mut::<Collider>(c).and_then(|col| {
                            col.collisions.clear();

                            col.active.then(|| (c, col.clone()))
                        })
                    })?,
                    cm.get_cache_id::<Transform>(e, em).and_then(|t| {
                        cm.get_cache::<Transform>(t).and_then(|transform| {
                            transform.active.then_some((t, transform.clone()))
                        })
                    })?,
                    cm.get::<Physical>(e, em).cloned(),
                ))
            })
            .collect::<Vec<_>>()
            .into_par_iter()
            .filter_map(|ref e @ (_, _, (_, ref b_transform), _)| {
                tree.lock()
                    .ok()?
                    .insert(b_transform.position(), e.clone())
                    .then_some(e.clone())
            })
            .collect();

        let checked = RwLock::new(Vec::new());

        if let Ok(tree) = tree.into_inner() {
            for ((ae, ac, at), (be, bc, bt), (ghost, (atr, btr))) in entities
                .par_iter()
                .cloned()
                .filter_map(|(ae, (ac, a_col), (at, a_transform), a_physical)| {
                    Some(
                        tree.query(Box2::new(a_transform.position(), a_col.boundary))
                            .into_iter()
                            .filter_map(|(_, t)| t)
                            .filter_map(|(be, (bc, b_col), (bt, b_transform), b_physical)| {
                                let res = {
                                    let res = {
                                        let checked = checked.read().ok()?;

                                        !checked.contains(&(ae, be)) && !checked.contains(&(be, ae))
                                    };
                                    if res {
                                        Some((
                                            (ae, ac, at),
                                            (be, bc, bt),
                                            Self::detect(
                                                (&a_col, &a_transform, &a_physical),
                                                (&b_col, &b_transform, &b_physical),
                                            )?,
                                        ))
                                    } else {
                                        None
                                    }
                                };

                                checked.write().ok()?.push((ae, be));

                                res
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .flatten()
                .collect::<Vec<_>>()
            {
                Self::resolve(ghost, ae, bc, bt, btr, cm);
                Self::resolve(ghost, be, ac, at, atr, cm);
            }
        }
    }
}

impl<'a> System<'a> for PhysicsManager {
    fn update(
        &mut self,
        ev: &mut Ev,
        _: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
            let now = Instant::now();
            let delta = now.duration_since(self.frame).min(self.max_delta);

            self.frame = now;

            for _ in 0..self.step_amount {
                for e in em.entities.keys().cloned() {
                    if let Some((pos, physical)) = cm
                        .get::<Physical>(e, em)
                        .cloned()
                        .and_then(|p| {
                            let force = p.active.then_some(p.force)?;
                            let t = cm.get_mut::<Transform>(e, em)?;
                            let pos = t.position();

                            t.set_position(
                                t.position()
                                    + force / self.step_amount as f32 * delta.as_secs_f32(),
                            );

                            self.check_collisions((em, cm));

                            Some(pos)
                        })
                        .and_then(|pos| Some((pos, cm.get_mut::<Physical>(e, em)?)))
                    {
                        if let Some(vel) = physical
                            .last_position()
                            .map(|l| (pos - l) / delta.as_secs_f32())
                        {
                            physical.set_velocity(vel);
                        }

                        physical.set_last_position(pos);
                    }
                }
            }
        }

        Ok(())
    }
}
