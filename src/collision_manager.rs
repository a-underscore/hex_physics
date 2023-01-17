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
            let collisions = {
                let mut objects: Vec<_> = world
                    .em
                    .entities
                    .keys()
                    .copied()
                    .filter_map(|e| {
                        world
                            .cm
                            .get_cached_id::<Transform>(e, &world.em)
                            .and_then(|p| {
                                Some((
                                    p,
                                    e,
                                    world
                                        .cm
                                        .get::<Collider>(e, &world.em)
                                        .and_then(|t| t.active.then_some(t))?,
                                    world
                                        .cm
                                        .get_cached::<Transform>(p)
                                        .and_then(|p| p.active.then_some(p))?,
                                ))
                            })
                    })
                    .collect();

                let mut collisions = Vec::new();

                while let Some((ac, ae, a, at)) = objects.pop() {
                    for (bc, be, b, bt) in &objects {
                        if let Some(v) = a.intersecting(at, b, bt) {
                            let a = (ac, ae);
                            let b = (*bc, *be);
                            let min_translation = (bt.position() - at.position()).normalize() * v;

                            if world.cm.get::<Physical>(ae, &world.em).is_some() {
                                collisions.push((-min_translation, a));
                            }

                            if world.cm.get::<Physical>(*be, &world.em).is_some() {
                                collisions.push((min_translation, b));
                            }
                        }
                    }
                }

                collisions
            };

            for (t, (ac, _)) in collisions {
                if let Some(p) = world.cm.get_cached_mut::<Transform>(ac) {
                    p.set_position(p.position() + t);
                }
            }
        }

        Ok(())
    }
}
