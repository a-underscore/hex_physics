use crate::polygon::Polygon;
use hex::{
    anyhow,
    components::Transform,
    ecs::{
        component_manager::ComponentManager,
        entity_manager::EntityManager,
        system_manager::{Ev, System},
    },
    glium::Display,
};

#[derive(Default)]
pub struct PhysicsManager;

impl<'a> System<'a> for PhysicsManager {
    fn update(
        &mut self,
        _: &Display,
        _: &mut Ev,
        entity_manager: &mut EntityManager,
        component_manager: &mut ComponentManager,
    ) -> anyhow::Result<()> {
        let collisions = {
            let mut objects: Vec<_> = entity_manager
                .entities
                .keys()
                .copied()
                .filter_map(|e| {
                    component_manager
                        .get_cached_val::<Polygon>(e, entity_manager)
                        .and_then(|p| {
                            Some((
                                p,
                                e,
                                component_manager.get_cached::<Polygon>(p)?,
                                component_manager.get::<Transform>(e, entity_manager)?,
                            ))
                        })
                })
                .collect();

            let mut collisions = Vec::new();

            while let Some((ac, ae, a, at)) = objects.pop() {
                for (bc, be, b, bt) in &objects {
                    if a.intersecting(at, b, bt) {
                        let a = (ac, ae);
                        let b = (*bc, *be);

                        collisions.extend([(a, b), (b, a)]);
                    }
                }
            }

            collisions
        };

        for ((ac, ae), (bc, be)) in collisions {
            if let Some(p) = component_manager.get_cached_mut::<Polygon>(ac) {
                p.collisions.push(be);
            }

            if let Some(p) = component_manager.get_cached_mut::<Polygon>(bc) {
                p.collisions.push(ae);
            }
        }

        Ok(())
    }
}
