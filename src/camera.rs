//====================================================================

use hecs::{Entity, World};
use hecs_engine::{
    common::{GlobalTransform, Size, Transform},
    engine::{tools::KeyCode, State},
    prelude::PerspectiveCamera,
};

//====================================================================

#[inline]
pub fn resize_camers(world: &mut World, size: Size<u32>) {
    world
        .query_mut::<&mut PerspectiveCamera>()
        .into_iter()
        .for_each(|(_, camera)| camera.aspect = size.width as f32 / size.height as f32);
}

pub struct FollowEntity {
    pub entity: Entity,
    pub damping: f32,
}

pub fn process_follow_entity(state: &mut State) {
    state
        .world()
        .query::<(&FollowEntity, &mut Transform)>()
        .into_iter()
        .for_each(|(_, (follow, transform))| {
            let target_global_transform = state
                .world()
                .get::<&GlobalTransform>(follow.entity)
                .unwrap();

            let (scale, rotation, translation) =
                target_global_transform.0.to_scale_rotation_translation();

            transform.scale = transform.scale.lerp(scale, follow.damping);
            transform.rotation = transform.rotation.lerp(rotation, follow.damping);
            transform.translation = transform.translation.lerp(translation, follow.damping);
        });
}

//====================================================================

const _CAMERA_MOVE_SPEED: f32 = 100.;

pub fn _debug_move_camera(state: &mut State) {
    let left = state.keys().pressed(KeyCode::KeyA);
    let right = state.keys().pressed(KeyCode::KeyD);

    let up = state.keys().pressed(KeyCode::Space);
    let down = state.keys().pressed(KeyCode::ShiftLeft);

    let forwards = state.keys().pressed(KeyCode::KeyW);
    let backwards = state.keys().pressed(KeyCode::KeyS);

    let x_dir = (right as i8 - left as i8) as f32;
    let y_dir = (up as i8 - down as i8) as f32;
    let z_dir = (forwards as i8 - backwards as i8) as f32;

    let dir = glam::Vec3::new(x_dir, y_dir, z_dir);

    //--------------------------------------------------

    let look_left = state.keys().pressed(KeyCode::KeyJ);
    let look_right = state.keys().pressed(KeyCode::KeyL);

    let look_up = state.keys().pressed(KeyCode::KeyI);
    let look_down = state.keys().pressed(KeyCode::KeyK);

    let yaw = (look_right as i8 - look_left as i8) as f32;
    let pitch = (look_down as i8 - look_up as i8) as f32;

    //--------------------------------------------------

    let delta = state.time().delta_seconds();

    let (_, (_, transform)) = state
        .world_mut()
        .query_mut::<(&PerspectiveCamera, &mut Transform)>()
        .into_iter()
        .next()
        .unwrap();

    if dir != glam::Vec3::ZERO {
        let forward = {
            let forward = transform.rotation * glam::Vec3::Z;
            glam::vec3(forward.x, 0., forward.z).normalize()
        };
        let right = {
            let right = transform.rotation * glam::Vec3::X;
            glam::vec3(right.x, 0., right.z).normalize()
        };
        let up = glam::Vec3::Y * dir.y;

        transform.translation += (forward + right + up) * _CAMERA_MOVE_SPEED * delta;
    }

    let yaw_rotation = glam::Quat::from_rotation_y(yaw);
    let pitch_rotation = glam::Quat::from_rotation_x(pitch);

    transform.rotation = yaw_rotation * transform.rotation * pitch_rotation;
}

//====================================================================
