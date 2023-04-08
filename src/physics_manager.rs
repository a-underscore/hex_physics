use crate::{Collider, Physical};
use hex::{
    anyhow,
    components::Transform,
    ecs::{ev::Control, system_manager::System, Ev, Id, Scene, World},
    glium::glutin::event::Event,
    math::Vec2,
};
use std::time::{Duration, Instant};

pub type Collision = (bool, (Option<Vec2>, Option<Vec2>));
pub type Colliders = Vec<(Id, (Id, Collider), Id, Option<Physical>)>;

pub struct PhysicsManager {
    pub step_amount: Id,
    pub max_delta: Duration,
    frame: Instant,
}

impl PhysicsManager {
    pub fn new(step_amount: Id, max_delta: Duration) -> Self {
        Self {
            step_amount,
            max_delta,
            frame: Instant::now(),
        }
    }

    pub fn detect(
        (ac, at, ap): (&Collider, Id, &Option<Physical>),
        (bc, bt, bp): (&Collider, Id, &Option<Physical>),
        world: &mut World,
    ) -> Option<Collision> {
        let at = world.cm.get_cache::<Transform>(at)?;
        let bt = world.cm.get_cache::<Transform>(bt)?;

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
        other_e: Id,
        cache_collider: Id,
        cache_transform: Id,
        tr: Option<Vec2>,
        world: &mut World,
    ) -> Option<()> {
        if let Some(collider) = world
            .cm
            .get_cache_mut::<Collider>(cache_collider)
            .and_then(|c| (!c.collisions.contains(&other_e)).then_some(c))
        {
            collider.collisions.push(other_e);
        }

        let (tr, t) = tr.and_then(|tr| {
            (!ghost_col).then_some((tr, world.cm.get_cache_mut::<Transform>(cache_transform)?))
        })?;

        t.set_position(t.position() + tr);

        Some(())
    }

    pub fn check_collisions(&mut self, mut entities: Colliders, world: &mut World) {
        while let Some((ae, (ac, a_col), at, a_physical)) = entities.pop() {
            for (be, (bc, b_col), bt, b_physical) in &entities {
                if let Some((ghost, (atr, btr))) =
                    Self::detect((&a_col, at, &a_physical), (b_col, *bt, b_physical), world)
                {
                    Self::resolve(ghost, ae, *bc, *bt, btr, world);
                    Self::resolve(ghost, *be, ac, at, atr, world);
                }
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
            let delta = now.duration_since(self.frame).min(self.max_delta);

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

                                    col.active.then(|| (c, col.clone()))
                                })
                            })?,
                        world
                            .cm
                            .get_cache_id::<Transform>(e, &world.em)
                            .and_then(|t| {
                                world
                                    .cm
                                    .get_cache::<Transform>(t)
                                    .and_then(|transform| transform.active.then_some(t))
                            })?,
                        world.cm.get::<Physical>(e, &world.em).cloned(),
                    ))
                })
                .collect();

            for _ in 0..self.step_amount {
                for e in world.em.entities.clone().into_keys() {
                    if let Some((pos, physical)) = world
                        .cm
                        .get_mut::<Physical>(e, &world.em)
                        .and_then(|p| p.active.then_some(p.force))
                        .and_then(|force| {
                            let t = world.cm.get_mut::<Transform>(e, &world.em)?;
                            let pos = t.position();

                            t.set_position(
                                t.position()
                                    + force / self.step_amount as f32 * delta.as_secs_f32(),
                            );

                            self.check_collisions(entities.clone(), world);

                            Some(pos)
                        })
                        .and_then(|pos| Some((pos, world.cm.get_mut::<Physical>(e, &world.em)?)))
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
