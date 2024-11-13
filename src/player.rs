//====================================================================

use hecs::Entity;
use hecs_engine::{
    common::{GlobalTransform, Transform},
    engine::{spatial::LocalTransform, tools::KeyCode, State},
    prelude::{PerspectiveCamera, Sprite},
};

use crate::physics::{Gravity, Velocity};

//====================================================================

pub struct Player;

//====================================================================

pub struct PlayerState {
    player: Entity,
    camera: Entity,
}

impl PlayerState {
    pub fn new(state: &mut State) -> Self {
        let texture = state.renderer().clone_default_texture();

        let player = state.world_mut().spawn((
            Player,
            Velocity::default(),
            Gravity::default(),
            Transform::from_translation((10., 0., -50.)),
            GlobalTransform::default(),
            Sprite {
                texture,
                size: glam::vec2(20., 20.),
                color: [0., 1., 0., 1.],
            },
        ));

        let camera = state.renderer().spawn_camera(PerspectiveCamera::default());

        state
            .world_mut()
            .insert(
                camera,
                (
                    LocalTransform {
                        parent: player,
                        transform: Transform::from_translation((0., 20., -50.)),
                    },
                    GlobalTransform::default(),
                ),
            )
            .unwrap();

        Self { player, camera }
    }

    const MOVE_SPEED: f32 = 100.;

    pub fn process_player(&self, state: &mut State) {
        let delta = state.time().delta_seconds();

        //--------------------------------------------------

        let mouse_motion = state.mouse_input().motion_delta() * delta;
        let yaw_rotation = glam::Quat::from_rotation_y(mouse_motion.x);

        // Apply to camera
        let pitch_rotation = glam::Quat::from_rotation_x(mouse_motion.y);

        let mut camera_local_transform = state
            .world()
            .get::<&mut LocalTransform>(self.camera)
            .unwrap();

        camera_local_transform.transform.rotation *= pitch_rotation;

        let camera_x_rotation = camera_local_transform
            .transform
            .rotation
            .to_euler(glam::EulerRot::XYZ)
            .0;

        let camera_rotation_x = camera_x_rotation.clamp(-45_f32.to_radians(), 45_f32.to_radians());

        camera_local_transform.transform.rotation = glam::Quat::from_rotation_x(camera_rotation_x);

        //--------------------------------------------------

        let left = state.keys().pressed(KeyCode::KeyA);
        let right = state.keys().pressed(KeyCode::KeyD);

        let forwards = state.keys().pressed(KeyCode::KeyW);
        let backwards = state.keys().pressed(KeyCode::KeyS);

        let x_dir = (right as i8 - left as i8) as f32;
        let z_dir = (forwards as i8 - backwards as i8) as f32;

        let move_dir = glam::Vec3::new(x_dir, 0., z_dir).normalize_or_zero();

        // let jump = state.keys().pressed(KeyCode::Space);
        // let sprint = state.keys().pressed(KeyCode::ShiftLeft);

        //--------------------------------------------------

        let player = state.world().entity(self.player).unwrap();

        let mut velocity = player.get::<&mut Velocity>().unwrap();
        let mut transform = player.get::<&mut Transform>().unwrap();
        // let gravity = player.get::<&Gravity>().unwrap();

        if move_dir != glam::Vec3::ZERO {
            let forward = {
                let forward = transform.rotation * glam::Vec3::Z;
                glam::vec3(forward.x, 0., forward.z).normalize()
            } * move_dir.z;

            let right = {
                let right = transform.rotation * glam::Vec3::X;
                glam::vec3(right.x, 0., right.z).normalize()
            } * move_dir.x;

            velocity.0 = (forward + right) * Self::MOVE_SPEED * delta;
        } else {
            velocity.0 = glam::Vec3::ZERO;
        }

        transform.rotation = yaw_rotation * transform.rotation;
    }
}

//====================================================================
