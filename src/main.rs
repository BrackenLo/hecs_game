//====================================================================

use hecs_engine::{
    common::GlobalTransform,
    pipelines::{texture_renderer::TextureRenderer, ui3d_renderer::Ui3dRenderer},
    prelude::*,
};
use physics::PhysicsHandler;
use player::PlayerState;

pub(crate) mod camera;
pub(crate) mod physics;
pub(crate) mod player;

//====================================================================

fn main() {
    Runner::<Game>::run();
}

pub struct Game {
    player_state: PlayerState,
    physics: PhysicsHandler,
}

impl App for Game {
    fn new(state: &mut State) -> Self {
        state
            .renderer()
            .add_renderer::<TextureRenderer>(5)
            .add_renderer::<Ui3dRenderer>(10);

        let default_texture = state.renderer().clone_default_texture();

        state.world_mut().spawn((
            Transform::default(),
            GlobalTransform::default(),
            Sprite {
                texture: default_texture,
                size: glam::vec2(100., 100.),
                color: [1., 0., 0., 1.],
            },
        ));

        let player_state = PlayerState::new(state);

        state.window().confine_cursor(true);

        let physics = PhysicsHandler::default();

        Self {
            player_state,
            physics,
        }
    }

    fn resize(&mut self, state: &mut State, size: Size<u32>) {
        camera::resize_camers(state.world_mut(), size);
    }

    fn update(&mut self, state: &mut State) {
        // camera::debug_move_camera(state);
        self.player_state.process_player(state);
        // self.player_state.debug_pos(state);

        self.physics.tick_physics(state);
    }
}

//====================================================================
