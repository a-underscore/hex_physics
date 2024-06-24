use crate::components::Collider;
use hex::{
    anyhow, components::Trans, system_manager::System, ComponentManager, Context, Control,
    EntityManager,
};
use std::sync::{Arc, RwLock};

pub struct PhysicsManager;

impl System for PhysicsManager {
    fn update(
        &mut self,
        _: Arc<RwLock<Control>>,
        _: Arc<RwLock<Context>>,
        em: Arc<RwLock<EntityManager>>,
        cm: Arc<RwLock<ComponentManager>>,
    ) -> anyhow::Result<()> {
        let em = em.read().unwrap();
        let cm = cm.read().unwrap();
        let mut entities: Vec<_> = em
            .entities()
            .filter_map(|e| Some((e, cm.get::<Trans>(e)?, cm.get::<Collider>(e)?)))
            .collect();

        while let Some((e, t, c)) = entities.pop() {
            let t = &mut *t.write().unwrap();
            let c = &mut *c.write().unwrap();

            for (e2, t2, c2) in &entities {
                let t2 = &mut *t2.write().unwrap();
                let c2 = &mut *c2.write().unwrap();

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
