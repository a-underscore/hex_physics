use crate::{collider::Collider, physical::Physical};
use hex::{
    anyhow,
    cgmath::InnerSpace,
    components::Transform,
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::glutin::event::Event,
};

#[derive(Default)]
pub struct CollisionManager;

impl<'a> System<'a> for CollisionManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Event::MainEventsCleared) = ev {
            let colliders: Vec<_> = world
                .em
                .entities
                .keys()
                .copied()
                .filter_map(|e| Some((e, world.cm.get_cached_id::<Collider>(e, &world.em)?)))
                .collect();

            for (_, ce) in colliders.iter().copied() {
                if let Some(c) = world.cm.get_cached_mut::<Collider>(ce) {
                    c.collisions.clear();
                }
            }

            let collisions = {
                let mut objects: Vec<_> = colliders
                    .into_iter()
                    .filter_map(|(e, ce)| {
                        world
                            .cm
                            .get_cached_id::<Transform>(e, &world.em)
                            .and_then(|te| {
                                Some((
                                    e,
                                    world
                                        .cm
                                        .get_cached::<Collider>(ce)
                                        .and_then(|c| c.active.then_some((ce, c)))?,
                                    world
                                        .cm
                                        .get_cached::<Transform>(te)
                                        .and_then(|t| t.active.then_some((te, t)))?,
                                ))
                            })
                    })
                    .collect();

                let mut collisions = Vec::new();

                while let Some((ae, (ace, ac), (ate, at))) = objects.pop() {
                    for (be, (bce, bc), (bte, bt)) in &objects {
                        if ac.layers.iter().find(|a| bc.layers.contains(a)).is_some() {
                            if let Some(v) = ac.intersecting(at, bc, bt) {
                                let a = (*be, ace, ate);
                                let b = (ae, *bce, *bte);
                                let min_translation =
                                    (bt.position() - at.position()).normalize() * v;

                                let ac = world
                                    .cm
                                    .get::<Physical>(ae, &world.em)
                                    .map(|_| -min_translation);
                                let bc = world
                                    .cm
                                    .get::<Physical>(*be, &world.em)
                                    .map(|_| min_translation);

                                collisions.extend([(ac, a), (bc, b)]);
                            }
                        }
                    }
                }

                collisions
            };

            for (tr, (be, c, t)) in collisions {
                if let Some(c) = world.cm.get_cached_mut::<Collider>(c) {
                    c.collisions.push(be);
                }

                if let Some((tr, t)) =
                    tr.and_then(|tr| Some((tr, world.cm.get_cached_mut::<Transform>(t)?)))
                {
                    t.set_position(t.position() + tr);
                }
            }
        }

        Ok(())
    }
}
