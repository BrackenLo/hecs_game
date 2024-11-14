//====================================================================

use std::sync::Arc;

use hecs::{Entity, EntityBuilder};
use hecs_engine::{
    pipelines::{texture_renderer::TextureRenderer, ui3d_renderer::Ui3dRenderer},
    prelude::*,
};
use physics::{CollisionShape, PhysicsHandler, StaticCollisionType};
use player::PlayerState;

pub(crate) mod camera;
pub(crate) mod physics;
pub(crate) mod player;

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

    _camera: Entity,
}

impl App for Game {
    fn new(state: &mut State) -> Self {
        state
            .renderer()
            .add_renderer::<TextureRenderer>(5)
            .add_renderer::<Ui3dRenderer>(10);

        state.window().confine_cursor(true);
        state.window().hide_cursor(true);

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
                camera::FollowEntity {
                    entity: player_state.camera_anchor(),
                    damping: 0.2,
                },
            )
            .unwrap();

        let physics = PhysicsHandler::default();

        Self {
            player_state,
            physics,
            _camera: camera,
        }
    }

    fn resize(&mut self, state: &mut State, size: Size<u32>) {
        camera::resize_camers(state.world_mut(), size);
    }

    fn update(&mut self, state: &mut State) {
        // camera::debug_move_camera(state);
        self.player_state.process_player(state);

        self.physics.tick_physics(state);

        camera::process_follow_entity(state);
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
