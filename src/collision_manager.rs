use crate::{collider::Collider, physical::Physical};
use hex::{
    anyhow,
    components::Transform,
    ecs::{ev::Control, system_manager::System, Ev, World},
    glium::glutin::event::Event,
    math::Vec2,
};

pub type Collision = (bool, (Option<Vec2>, Option<Vec2>));

#[derive(Default)]
pub struct CollisionManager;

impl CollisionManager {
    pub fn detect(
        ae: usize,
        cache_ac: usize,
        cache_at: usize,
        be: usize,
        cache_bc: usize,
        cache_bt: usize,
        world: &mut World,
    ) -> Option<Collision> {
        let ac = world.cm.get_cache::<Collider>(cache_ac)?;
        let at = world.cm.get_cache::<Transform>(cache_at)?;
        let bc = world.cm.get_cache::<Collider>(cache_bc)?;
        let bt = world.cm.get_cache::<Transform>(cache_bt)?;

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

                return Some((ac.ghost || bc.ghost, (act, bct)));
            }
        }

        None
    }

    pub fn resolve(
        ghost_col: bool,
        other_e: usize,
        cache_collider: usize,
        cache_transform: usize,
        tr: Option<Vec2>,
        world: &mut World,
    ) {
        if let Some(c) = world.cm.get_cache_mut::<Collider>(cache_collider) {
            c.collisions.push(other_e);
        }

        if let Some((tr, t)) = tr
            .and_then(|tr| Some((tr, world.cm.get_cache_mut::<Transform>(cache_transform)?)))
            .filter(|_| !ghost_col)
        {
            t.set_position(t.position() + tr);
        }
    }
}

impl<'a> System<'a> for CollisionManager {
    fn update(&mut self, ev: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Event(Control {
            event: Event::MainEventsCleared,
            flow: _,
        }) = ev
        {
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
                            .get_cache_id::<Collider>(e, &world.em)
                            .and_then(|c| {
                                world.cm.get_cache_mut::<Collider>(c).and_then(|col| {
                                    col.collisions.clear();

                                    col.active.then_some(c)
                                })
                            })?,
                        world
                            .cm
                            .get_cache_id::<Transform>(e, &world.em)
                            .and_then(|t| {
                                world.cm.get_cache::<Transform>(t)?.active.then_some(t)
                            })?,
                    ))
                })
                .collect();

            while let Some((ae, ac, at)) = objects.pop() {
                for (be, bc, bt) in objects.iter().cloned() {
                    if let Some((ghost, (atr, btr))) = Self::detect(ae, ac, at, be, bc, bt, world) {
                        Self::resolve(ghost, ae, bc, bt, btr, world);
                        Self::resolve(ghost, be, ac, at, atr, world);
                    }
                }
            }
        }

        Ok(())
    }
}
