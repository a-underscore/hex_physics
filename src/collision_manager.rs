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
                if let Some(velocity) = world
                    .component_manager
                    .get_mut::<Momentum>(e, &world.entity_manager)
                    .map(|m| {
                        let applied_m = m.applied.clone();

                        m.applied.clear();

                        (Into::<Vector2<f32>>::into(m.clone()), applied_m)
                    })
                    .map(|(mut m, applied_m)| {
                        for e in applied_m {
                            if let Some(m2) = world
                                .component_manager
                                .get::<Momentum>(e, &world.entity_manager)
                            {
                                m += m2.clone().into();
                            }
                        }

                        m
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
