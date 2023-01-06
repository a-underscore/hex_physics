use crate::force::Force;
use hex::{
    anyhow,
    components::Transform,
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::glutin::event::Event,
};
use std::time::Instant;

pub struct ForceManager {
    frame: Instant,
}

impl Default for ForceManager {
    fn default() -> Self {
        Self {
            frame: Instant::now(),
        }
    }
}

impl<'a> System<'a> for ForceManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Event::MainEventsCleared) = ev {
            let now = Instant::now();
            let delta = now.duration_since(self.frame);

            self.frame = now;

            for e in world.entity_manager.entities.keys().copied() {
                world
                    .component_manager
                    .get::<Force>(e, &world.entity_manager)
                    .map(|f| f.velocity)
                    .map(|p| {
                        world
                            .component_manager
                            .get_mut::<Transform>(e, &world.entity_manager)
                            .map(|t| t.set_position(t.position() + p * delta.as_secs_f32()));
                    });
            }
        }

        Ok(())
    }
}
