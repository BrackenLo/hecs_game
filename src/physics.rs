//====================================================================

use hecs_engine::{common::Transform, engine::State};

//====================================================================

#[derive(Default)]
pub struct Velocity(pub glam::Vec3);

pub struct Gravity(pub glam::Vec3);
impl Default for Gravity {
    #[inline]
    fn default() -> Self {
        Self(glam::vec3(0., -5., 0.))
    }
}

pub struct Grounded;

//====================================================================

pub enum CollisionShape {
    Box {
        half_width: f32,
        half_height: f32,
        half_depth: f32,
    },
    Plane {
        half_width: f32,
        half_depth: f32,
    },
}

//====================================================================

#[derive(Default)]
pub struct PhysicsHandler {}

impl PhysicsHandler {
    pub fn tick_physics(&mut self, state: &mut State) {
        state
            .world_mut()
            .query_mut::<(&mut Transform, &Velocity)>()
            .into_iter()
            .for_each(|(_, (transform, velocity))| {
                transform.translation += velocity.0;
            });
    }
}

//====================================================================
