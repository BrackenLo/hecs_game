//====================================================================

use std::sync::Arc;

use hecs::{Entity, EntityBuilder};
use hecs_engine::{
    engine::tools::KeyCode,
    pipelines::{texture_renderer::TextureRenderer, ui3d_renderer::Ui3dRenderer},
    prelude::*,
};
use physics::{CollisionShape, PhysicsHandler, StaticCollisionType};
use player::PlayerState;

pub(crate) mod camera;
pub(crate) mod physics;
pub(crate) mod player;
pub(crate) mod tools;

//====================================================================

fn main() {
    env_logger::Builder::new()
        .filter_module("hecs_game", log::LevelFilter::Trace)
        .filter_module("hecs_engine", log::LevelFilter::Trace)
        //
        .filter_module("engine", log::LevelFilter::Trace)
        .filter_module("pipelines", log::LevelFilter::Trace)
        .filter_module("renderer", log::LevelFilter::Trace)
        .filter_module("common", log::LevelFilter::Trace)
        //
        .filter_module("winit", log::LevelFilter::Trace)
        .filter_module("wgpu", log::LevelFilter::Warn)
        .init();

    log::info!("Hello World");

    Runner::<Game>::run();
}

pub struct Game {
    player_state: PlayerState,
    physics: PhysicsHandler,

    camera: Entity,

    cursor_locked: bool,
    camera_debug: bool,
}

impl App for Game {
    fn new(state: &mut State) -> Self {
        state
            .renderer()
            .add_renderer::<TextureRenderer>(5)
            .add_renderer::<Ui3dRenderer>(10);

        let cursor_locked = true;
        state.window().confine_cursor(cursor_locked);
        state.window().hide_cursor(cursor_locked);

        let default_texture = state.renderer().clone_default_texture();

        spawn_floor(state.world_mut(), default_texture.clone());

        state.world_mut().spawn((
            Transform::default(),
            GlobalTransform::default(),
            Sprite {
                texture: default_texture,
                half_size: glam::vec2(100., 100.),
                color: [1., 0., 0., 1.],
            },
        ));

        let player_state = PlayerState::new(state);
        let camera = spawn_camera(state);
        state
            .world_mut()
            .insert_one(
                camera,
                tools::FollowEntity {
                    entity: player_state.camera_anchor(),
                    damping: 0.2,
                },
            )
            .unwrap();

        let physics = PhysicsHandler::default();

        Self {
            player_state,
            physics,
            camera,
            cursor_locked,
            camera_debug: false,
        }
    }

    fn resize(&mut self, state: &mut State, size: Size<u32>) {
        camera::resize_camers(state.world_mut(), size);
    }

    fn update(&mut self, state: &mut State) {
        if state.keys().just_pressed(KeyCode::F1) {
            self.cursor_locked = !self.cursor_locked;
            state.window().confine_cursor(self.cursor_locked);
            state.window().hide_cursor(self.cursor_locked);
        }

        if state.keys().just_pressed(KeyCode::F2) {
            self.camera_debug = !self.camera_debug;
            self.player_state.movement_disabled = self.camera_debug;

            match self.camera_debug {
                true => {
                    state
                        .world_mut()
                        .remove_one::<tools::FollowEntity>(self.camera)
                        .unwrap();
                }
                false => state
                    .world_mut()
                    .insert_one(
                        self.camera,
                        tools::FollowEntity {
                            entity: self.player_state.camera_anchor(),
                            damping: 0.2,
                        },
                    )
                    .unwrap(),
            };
        }

        if self.camera_debug {
            camera::debug_move_camera(state);
        }

        // camera::debug_move_camera(state);
        self.player_state.process_player(state);

        self.physics.tick_physics(state);

        tools::process_follow_entity(state);
    }
}

//====================================================================

fn spawn_camera(state: &mut State) -> Entity {
    let mut builder = EntityBuilder::new();
    builder
        .add(GlobalTransform::default())
        .add(Transform::default());

    state
        .renderer()
        .spawn_camera(&mut builder, PerspectiveCamera::default());

    state.world_mut().spawn(builder.build())
}

fn spawn_floor(world: &mut hecs::World, texture: Arc<LoadedTexture>) {
    world.spawn((
        Transform::from_rotation_translation(
            glam::Quat::from_rotation_x(90_f32.to_radians()),
            glam::vec3(0., -40., 0.),
        ),
        GlobalTransform::default(),
        Sprite {
            texture,
            half_size: glam::vec2(500., 500.),
            color: [0.3, 0.3, 0.3, 1.],
        },
        StaticCollisionType,
        CollisionShape::Box {
            half_width: 500.,
            half_height: 5.,
            half_depth: 500.,
        },
    ));
}

//====================================================================
