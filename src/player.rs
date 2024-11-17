//====================================================================

use hecs::{Entity, EntityBuilder};
use hecs_engine::{
    common::{GlobalTransform, Transform},
    engine::{spatial::LocalTransform, tools::KeyCode, State},
    pipelines::model_renderer::Model,
};

use crate::{
    physics::{
        Accel, CharacterCollisionBundle, CharacterController, CharacterMovementBundle,
        CollisionShape,
    },
    Resources,
};

//====================================================================

pub struct Player;

pub struct PlayerSpeed {
    walk: f32,
    sprint: f32,
}

//====================================================================

pub struct PlayerState {
    player: Entity,
    pub movement_disabled: bool,

    camera_anchor: Entity,
    camera_angle: f32,
    camera_distance: f32,
    camera_zoom_speed: f32,
}

impl PlayerState {
    pub fn new(state: &mut State, resources: &Resources) -> Self {
        let player = state.world_mut().spawn(
            EntityBuilder::new()
                .add(Player)
                .add(PlayerSpeed {
                    walk: 1300.,
                    sprint: 2000.,
                })
                .add(Transform::from_translation((10., 0., -200.)))
                .add(GlobalTransform::default())
                // .add(Sprite {
                //     texture,
                //     size: glam::vec2(20., 35.),
                //     color: [0., 1., 0., 1.],
                // })
                .add(Model {
                    meshes: vec![(resources.cube.clone(), resources.texture.clone())],
                    color: [0., 1., 0., 1.],
                    scale: glam::vec3(20., 40., 20.),
                })
                .add_bundle(CharacterMovementBundle::default())
                .add_bundle(CharacterCollisionBundle::from_shape(CollisionShape::Box {
                    half_width: 10.,
                    half_height: 20.,
                    half_depth: 10.,
                }))
                .build(),
        );

        let camera_pos = glam::vec2(35., -50.);
        let camera_angle = camera_pos.to_angle();
        let camera_distance = camera_pos.length();
        let camera_zoom_speed = 4.;

        let camera_anchor = state.world_mut().spawn((
            LocalTransform {
                parent: player,
                transform: Transform::from_translation((0., camera_pos.x, camera_pos.y)),
            },
            GlobalTransform::default(),
        ));

        Self {
            player,
            movement_disabled: false,

            camera_anchor,
            camera_angle,
            camera_distance,
            camera_zoom_speed,
        }
    }

    pub fn process_player(&mut self, state: &mut State) {
        if self.movement_disabled {
            return;
        }

        let delta = state.time().delta_seconds();

        //--------------------------------------------------

        let mouse_motion = state.mouse_input().motion_delta() * delta;
        let yaw_rotation = glam::Quat::from_rotation_y(mouse_motion.x);

        // Apply to camera
        let pitch_rotation = glam::Quat::from_rotation_x(mouse_motion.y);

        let mut camera_local_transform = state
            .world()
            .get::<&mut LocalTransform>(self.camera_anchor)
            .unwrap();

        camera_local_transform.transform.rotation *= pitch_rotation;

        let camera_x_rotation = camera_local_transform
            .transform
            .rotation
            .to_euler(glam::EulerRot::XYZ)
            .0;

        let camera_rotation_x = camera_x_rotation.clamp(-45_f32.to_radians(), 45_f32.to_radians());

        camera_local_transform.transform.rotation = glam::Quat::from_rotation_x(camera_rotation_x);

        let scroll = state.mouse_input().scroll().y;
        if scroll != 0. {
            self.camera_distance -= scroll * self.camera_zoom_speed;
            self.camera_distance = self.camera_distance.clamp(20., 200.);

            let new_camera_pos = glam::Vec2::from_angle(self.camera_angle) * self.camera_distance;
            camera_local_transform.transform.translation.y = new_camera_pos.x;
            camera_local_transform.transform.translation.z = new_camera_pos.y;
        }

        //--------------------------------------------------

        let left = state.keys().pressed(KeyCode::KeyA);
        let right = state.keys().pressed(KeyCode::KeyD);

        let forwards = state.keys().pressed(KeyCode::KeyW);
        let backwards = state.keys().pressed(KeyCode::KeyS);

        let x_dir = (right as i8 - left as i8) as f32;
        let z_dir = (forwards as i8 - backwards as i8) as f32;

        let move_dir = glam::Vec3::new(x_dir, 0., z_dir).normalize_or_zero();

        let jump = state.keys().pressed(KeyCode::Space);
        let sprint = state.keys().pressed(KeyCode::ShiftLeft);

        //--------------------------------------------------

        state
            .world()
            .get::<&CharacterController>(self.player)
            .unwrap();

        let player = state.world().entity(self.player).unwrap();

        let mut controller = player.get::<&mut CharacterController>().unwrap();
        let mut transform = player.get::<&mut Transform>().unwrap();

        if move_dir != glam::Vec3::ZERO {
            let forward = {
                let forward = transform.rotation * glam::Vec3::Z;
                glam::vec2(forward.x, forward.z).normalize()
            } * move_dir.z;

            let right = {
                let right = transform.rotation * glam::Vec3::X;
                glam::vec2(right.x, right.z).normalize()
            } * move_dir.x;

            let direction = forward + right;

            controller
                .movement_action_queue
                .push(crate::physics::MovementAction::Move(direction.into()));
        }

        transform.rotation = yaw_rotation * transform.rotation;

        if jump {
            controller
                .movement_action_queue
                .push(crate::physics::MovementAction::Jump);
        }

        let mut accel = player.get::<&mut Accel>().unwrap();
        let speed = player.get::<&PlayerSpeed>().unwrap();

        accel.0 = match sprint {
            true => speed.sprint,
            false => speed.walk,
        };
    }

    #[inline]
    pub fn _player(&self) -> Entity {
        self.player
    }

    #[inline]
    pub fn camera_anchor(&self) -> Entity {
        self.camera_anchor
    }
}

//====================================================================
