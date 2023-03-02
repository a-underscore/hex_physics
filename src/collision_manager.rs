use crate::{collider::Collider, physical::Physical};
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

pub type Collision = (bool, (Option<Vector2<f32>>, Option<Vector2<f32>>));

#[derive(Default)]
pub struct CollisionManager;

impl CollisionManager {
    pub fn detect(
        ae: usize,
        cached_ac: usize,
        cached_at: usize,
        be: usize,
        cached_bc: usize,
        cached_bt: usize,
        world: &mut World,
    ) -> Option<Collision> {
        let ac = world.cm.get_cached::<Collider>(cached_ac)?;
        let at = world.cm.get_cached::<Transform>(cached_at)?;
        let bc = world.cm.get_cached::<Collider>(cached_bc)?;
        let bt = world.cm.get_cached::<Transform>(cached_bt)?;

        if ac.layers.iter().any(|a| bc.layers.contains(a))
            && !ac.ignore.iter().any(|a| bc.layers.contains(a))
            && !bc.ignore.iter().any(|b| ac.layers.contains(b))
        {
            if let Some(min_translation) = ac.intersecting(at, bc, bt) {
                let act = world
                    .cm
                    .get::<Physical>(ae, &world.em)
                    .map(|_| -min_translation);
                let bct = world
                    .cm
                    .get::<Physical>(be, &world.em)
                    .map(|_| min_translation);

                return Some((ac.ray || bc.ray, (act, bct)));
            }
        }

        None
    }

    pub fn resolve(
        ray_col: bool,
        other_e: usize,
        cached_collider: usize,
        cached_transform: usize,
        tr: Option<Vector2<f32>>,
        world: &mut World,
    ) {
        if let Some(c) = world.cm.get_cached_mut::<Collider>(cached_collider) {
            c.collisions.push(other_e);
        }

        if let Some((tr, t)) = tr
            .and_then(|tr| Some((tr, world.cm.get_cached_mut::<Transform>(cached_transform)?)))
            .filter(|_| !ray_col)
        {
            t.set_position(t.position() + tr);
        }
    }
}

impl<'a> System<'a> for CollisionManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Event::MainEventsCleared) = ev {
            let mut objects: Vec<_> = world
                .em
                .entities
                .keys()
                .cloned()
                .filter_map(|e| {
                    Some((
                        e,
                        world
                            .cm
                            .get_cached_id::<Collider>(e, &world.em)
                            .and_then(|c| {
                                world.cm.get_cached_mut::<Collider>(c).and_then(|col| {
                                    col.collisions.clear();

                                    col.active.then_some(c)
                                })
                            })?,
                        world
                            .cm
                            .get_cached_id::<Transform>(e, &world.em)
                            .and_then(|t| {
                                world.cm.get_cached::<Transform>(t)?.active.then_some(t)
                            })?,
                    ))
                })
                .collect();

            while let Some((ae, ac, at)) = objects.pop() {
                for (be, bc, bt) in objects.iter().cloned() {
                    if let Some((ray, (atr, btr))) = Self::detect(ae, ac, at, be, bc, bt, world) {
                        Self::resolve(ray, ae, bc, bt, btr, world);
                        Self::resolve(ray, be, ac, at, atr, world);
                    }
                }
            }
        }

        Ok(())
    }
}
