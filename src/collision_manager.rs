use crate::{collider::Collider, physical::Physical};
use hex::{
    anyhow,
    cgmath::{InnerSpace, Vector2},
    components::Transform,
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::glutin::event::Event,
};

#[derive(Default)]
pub struct CollisionManager;

impl CollisionManager {
    pub fn detect(
        ae: usize,
        cached_ac: usize,
        cached_at: usize,
        be: usize,
        cached_bc: usize,
        cached_bt: usize,
        world: &mut World,
    ) -> Option<(
        (usize, usize, usize, Option<Vector2<f32>>),
        (usize, usize, usize, Option<Vector2<f32>>),
    )> {
        let ac = world
            .cm
            .get_cached::<Collider>(cached_ac)
            .and_then(|c| c.active.then_some(c))?;
        let at = world
            .cm
            .get_cached::<Transform>(cached_at)
            .and_then(|t| t.active.then_some(t))?;
        let bc = world
            .cm
            .get_cached::<Collider>(cached_bc)
            .and_then(|c| c.active.then_some(c))?;
        let bt = world
            .cm
            .get_cached::<Transform>(cached_bt)
            .and_then(|t| t.active.then_some(t))?;

        if ac.layers.iter().any(|a| bc.layers.contains(a)) {
            if let Some(v) = ac.intersecting(at, bc, bt) {
                let min_translation = (bt.position() - at.position()).normalize() * v;

                let ac = world
                    .cm
                    .get::<Physical>(ae, &world.em)
                    .map(|_| -min_translation);
                let bc = world
                    .cm
                    .get::<Physical>(be, &world.em)
                    .map(|_| min_translation);

                return Some((
                    (be, cached_ac, cached_at, ac),
                    (ae, cached_bc, cached_bt, bc),
                ));
            }
        }

        None
    }

    pub fn insert(
        other_e: usize,
        cached_collider: usize,
        cached_transform: usize,
        tr: Option<Vector2<f32>>,
        world: &mut World,
    ) {
        if let Some(c) = world.cm.get_cached_mut::<Collider>(cached_collider) {
            c.collisions.push(other_e);
        }

        if let Some((tr, t)) =
            tr.and_then(|tr| Some((tr, world.cm.get_cached_mut::<Transform>(cached_transform)?)))
        {
            t.set_position(t.position() + tr);
        }
    }
}

impl<'a> System<'a> for CollisionManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Event::MainEventsCleared) = ev {
            let mut objects: Vec<_> = world
                .em
                .entities
                .keys()
                .cloned()
                .filter_map(|e| {
                    Some((
                        e,
                        world
                            .cm
                            .get_cached_id::<Collider>(e, &world.em)
                            .and_then(|c| {
                                world.cm.get_cached::<Collider>(c)?.active.then_some(c)
                            })?,
                        world
                            .cm
                            .get_cached_id::<Transform>(e, &world.em)
                            .and_then(|t| {
                                world.cm.get_cached::<Transform>(t)?.active.then_some(t)
                            })?,
                    ))
                })
                .collect();

            while let Some((ae, ac, at)) = objects.pop() {
                for (be, bc, bt) in objects.iter().cloned() {
                    if let Some(((ae, at, ac, atr), (be, bt, bc, btr))) =
                        Self::detect(ae, ac, at, be, bc, bt, world)
                    {
                        Self::insert(be, at, ac, atr, world);
                        Self::insert(ae, bt, bc, btr, world);
                    }
                }
            }
        }

        Ok(())
    }
}
