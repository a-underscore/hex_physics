use crate::physical::Physical;
use hex::{
    anyhow,
    components::Transform,
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::glutin::event::Event,
};
use std::time::{Duration, Instant};

pub struct PhysicsManager {
    frame: Instant,
    pub max_delta: Duration,
}

impl PhysicsManager {
    pub fn new(max_delta: Duration) -> Self {
        Self {
            frame: Instant::now(),
            max_delta,
        }
    }
}

impl<'a> System<'a> for PhysicsManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Event::MainEventsCleared) = ev {
            let now = Instant::now();
            let delta = now.duration_since(self.frame);

            self.frame = now;

            for e in world.em.entities.keys().copied() {
                if let Some(velocity) = world
                    .cm
                    .get_mut::<Physical>(e, &world.em)
                    .and_then(|p| p.active.then_some(p.velocity))
                {
                    if let Some(t) = world.cm.get_mut::<Transform>(e, &world.em) {
                        t.set_position(
                            t.position() + velocity * (delta.min(self.max_delta)).as_secs_f32(),
                        )
                    }
                }
            }
        }

        Ok(())
    }
}
