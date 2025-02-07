use crate::components::Collider;
use hex::{
    anyhow,
    components::Trans,
    parking_lot::RwLock,
    world::{system_manager::System, World},
    Context, Control,
};
use std::sync::Arc;

pub struct PhysicsManager;

impl System for PhysicsManager {
    fn update(
        &mut self,
        _: Arc<RwLock<Control>>,
        _: Arc<RwLock<Context>>,
        world: Arc<RwLock<World>>,
    ) -> anyhow::Result<()> {
        let em = world.read().em.clone();
        let em = em.read();
        let mut entities: Vec<_> = em
            .entities()
            .filter_map(|e| {
                Some((
                    e,
                    em.get_component::<Collider>(e)?,
                    em.get_component::<Trans>(e)?,
                ))
            })
            .collect();

        while let Some((e, c, t)) = entities.pop() {
            let t = &mut *t.write();
            let c = &mut *c.write();

            for (e2, c2, t2) in &entities {
                let c2 = &mut *c2.write();
                let t2 = &mut *t2.write();

                if c.layers.iter().any(|a| c2.layers.contains(a))
                    && !(c.ignore.iter().any(|a| c2.layers.contains(a))
                        || c2.ignore.iter().any(|b| c.layers.contains(b)))
                    && (t.position() - t2.position()).magnitude() <= c.boundary + c2.boundary
                {
                    if let Some(res) = c.intersecting(&t, &t2, &c2) {
                        if !(c.ghost || c2.ghost) {
                            t.set_position(t.position() + res);
                            t2.set_position(t2.position() - res);

                            if c.log_collisions && !c.collisions.contains(e2) {
                                c.collisions.push(*e2);
                            }

                            if c2.log_collisions && !c2.collisions.contains(&e) {
                                c2.collisions.push(e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl System for PhysicsManager {}
