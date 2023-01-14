use crate::physical::Physical;
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

pub const MAX_DELTA: f32 = 1.0;

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
                    .get_mut::<Physical>(e, &world.entity_manager)
                    .map(|p| {
                        let applied = p.applied.clone();

                        p.applied.clear();

                        p.velocity + applied.into_iter().sum::<Vector2<f32>>()
                    })
                {
                    if let Some(t) = world
                        .component_manager
                        .get_mut::<Transform>(e, &world.entity_manager)
                    {
                        t.set_position(t.position() + velocity * delta.as_secs_f32().min(MAX_DELTA))
                    }
                }
            }
        }

        Ok(())
    }
}
