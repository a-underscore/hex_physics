use crate::collider::Collider;
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
                    .entity_manager
                    .entities
                    .keys()
                    .copied()
                    .filter_map(|e| {
                        world
                            .component_manager
                            .get_cached_id::<Collider>(e, &world.entity_manager)
                            .and_then(|p| {
                                Some((
                                    p,
                                    e,
                                    world
                                        .component_manager
                                        .get_cached::<Collider>(p)
                                        .and_then(|p| p.active.then_some(p))?,
                                    world
                                        .component_manager
                                        .get::<Transform>(e, &world.entity_manager)
                                        .and_then(|t| t.active.then_some(t))?,
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

                            collisions.extend([(min_translation, a, b), (-min_translation, b, a)]);
                        }
                    }
                }

                collisions
            };

            for (t, (ac, ae), (bc, be)) in collisions {
                if let Some(p) = world.component_manager.get_cached_mut::<Collider>(ac) {
                    p.collisions.push((be, t));
                }

                if let Some(p) = world.component_manager.get_cached_mut::<Collider>(bc) {
                    p.collisions.push((ae, t));
                }
            }
        }

        Ok(())
    }
}
