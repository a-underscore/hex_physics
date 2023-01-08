use crate::momentum::Momentum;
use hex::{
    anyhow,
    cgmath::Vector2,
    components::Transform,
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::glutin::event::Event,
};
use std::time::Instant;

pub struct PhysicsManager {
    frame: Instant,
}

impl Default for PhysicsManager {
    fn default() -> Self {
        Self {
            frame: Instant::now(),
        }
    }
}

impl<'a> System<'a> for PhysicsManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Event::MainEventsCleared) = ev {
            let now = Instant::now();
            let delta = now.duration_since(self.frame);

            self.frame = now;

            for e in world.entity_manager.entities.keys().copied() {
                if let Some(velocity) = world
                    .component_manager
                    .get_mut::<Momentum>(e, &world.entity_manager)
                    .map(|m| {
                        let applied_m = m.applied.clone();

                        m.applied.clear();

                        (m.clone(), m.mass, applied_m)
                    })
                    .map(|(momentum, mass, applied_m)| {
                        let momentum: Vector2<f32> = momentum.into();
                        let total: Vector2<f32> = applied_m
                            .into_iter()
                            .filter_map(|m| {
                                let m = world
                                    .component_manager
                                    .get::<Momentum>(m, &world.entity_manager)?;
                                let m = (Into::<Vector2<f32>>::into(m.clone()) + momentum)
                                    / (m.mass + mass);

                                Some(m)
                            })
                            .sum::<Vector2<f32>>();

                        total
                    })
                {
                    if let Some(t) = world
                        .component_manager
                        .get_mut::<Transform>(e, &world.entity_manager)
                    {
                        t.set_position(t.position() + velocity * delta.as_secs_f32())
                    }
                }
            }
        }

        Ok(())
    }
}
