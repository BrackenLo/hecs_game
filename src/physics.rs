//====================================================================

use hecs::{Bundle, ChangeTracker, Entity, World};
use hecs_engine::{common::Transform, engine::State};

//====================================================================

pub enum MovementAction {
    Move((f32, f32)),
    Jump,
}

//====================================================================

pub struct CharacterController {
    pub movement_action_queue: Vec<MovementAction>,
}

pub struct Velocity(pub glam::Vec3);
pub struct Accel(pub f32);
pub struct DeaccelDampingFactor(pub f32);
pub struct JumpImpulse(pub f32);
pub struct Gravity(pub glam::Vec3);

pub struct Grounded;

#[derive(Bundle)]
pub struct CharacterMovementBundle {
    controller: CharacterController,
    velocity: Velocity,
    accel: Accel,
    deaccel: DeaccelDampingFactor,
    jump: JumpImpulse,
    gravity: Gravity,
}

impl Default for CharacterMovementBundle {
    fn default() -> Self {
        Self {
            controller: CharacterController {
                movement_action_queue: Vec::new(),
            },
            velocity: Velocity(glam::Vec3::ZERO),
            accel: Accel(1300.),
            deaccel: DeaccelDampingFactor(0.9),
            jump: JumpImpulse(200.),
            gravity: Gravity(glam::vec3(0., -400., 0.)),
        }
    }
}

//====================================================================

#[derive(Default)]
pub struct CollisionHits {
    hits: Vec<(Entity, CollisionDirection)>,
}

pub enum CollisionShape {
    Box {
        half_width: f32,
        half_height: f32,
        half_depth: f32,
    },
}

//--------------------------------------------------

pub struct DynamicCollisionType;

#[derive(Bundle)]
pub struct CharacterCollisionBundle {
    hits: CollisionHits,
    dynamic: DynamicCollisionType,
    shape: CollisionShape,
}

impl Default for CharacterCollisionBundle {
    fn default() -> Self {
        Self {
            hits: CollisionHits::default(),
            dynamic: DynamicCollisionType,
            shape: CollisionShape::Box {
                half_width: 20.,
                half_height: 20.,
                half_depth: 20.,
            },
        }
    }
}

//====================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticCollisionType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerCollisionType;

//====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionDirection {
    Horizontal,
    VerticalTop,
    VerticalBottom,
}

//====================================================================

#[derive(Default)]
pub struct PhysicsHandler {
    static_tracker: ChangeTracker<StaticCollisionType>,
}

impl PhysicsHandler {
    pub fn tick_physics(&mut self, state: &mut State) {
        prep_new_static(state, &mut self.static_tracker);
        clear_collision_hits(state);

        apply_character_movement(state);
        apply_deacceleration(state);
        apply_gravity(state);

        apply_velocity_collisions(state);
        update_grounded(state);

        clear_controller_actions(state);
    }
}

//====================================================================

fn prep_new_static(state: &mut State, tracker: &mut ChangeTracker<StaticCollisionType>) {
    let new_static = tracker
        .track(state.world_mut())
        .added()
        .into_iter()
        .map(|(e, _)| e)
        .collect::<Vec<_>>();

    new_static.into_iter().for_each(|entity| {
        let prepped = {
            let entity = state.world().entity(entity).unwrap();
            let transform = entity.get::<&Transform>().unwrap();
            let shape = entity.get::<&CollisionShape>().unwrap();

            PreppedCollisionShape::from_collision_shape(
                &shape,
                transform.translation,
                transform.scale,
            )
        };

        state.world_mut().insert_one(entity, prepped).unwrap();
    });
}

#[inline]
fn clear_collision_hits(state: &mut State) {
    state
        .world_mut()
        .query_mut::<&mut CollisionHits>()
        .into_iter()
        .for_each(|(_, hits)| hits.hits.clear());
}

fn apply_character_movement(state: &mut State) {
    let delta_time = state.time().delta_seconds();

    state
        .world_mut()
        .query_mut::<(
            &CharacterController,
            &Accel,
            Option<&JumpImpulse>,
            &mut Velocity,
            Option<&Grounded>,
        )>()
        .into_iter()
        .for_each(|(_, (controller, accel, jump, velocity, grounded))| {
            controller
                .movement_action_queue
                .iter()
                .for_each(|action| match action {
                    MovementAction::Move((x_dir, z_dir)) => {
                        velocity.0.x += x_dir * accel.0 * delta_time;
                        velocity.0.z += z_dir * accel.0 * delta_time;
                    }

                    MovementAction::Jump => match (jump, grounded) {
                        (Some(jump), Some(_)) => velocity.0.y = jump.0,
                        _ => {}
                    },
                });
        });
}

#[inline]
fn apply_deacceleration(state: &mut State) {
    state
        .world_mut()
        .query_mut::<(&mut Velocity, &DeaccelDampingFactor)>()
        .into_iter()
        .for_each(|(_, (velocity, damping))| {
            velocity.0.x *= damping.0;
            velocity.0.z *= damping.0;
        });
}

#[inline]
fn apply_gravity(state: &mut State) {
    let delta_time = state.time().delta_seconds();

    state
        .world_mut()
        .query_mut::<(&Gravity, &mut Velocity)>()
        .into_iter()
        .for_each(|(_, (gravity, velocity))| velocity.0 += gravity.0 * delta_time);
}

fn apply_velocity_collisions(state: &mut State) {
    let delta_time = state.time().delta_seconds();

    state
        .world()
        .query::<(
            &mut Transform,
            &mut Velocity,
            &CollisionShape,
            Option<&mut CollisionHits>,
        )>()
        .with::<&DynamicCollisionType>()
        .without::<(&StaticCollisionType, &TriggerCollisionType)>()
        .into_iter()
        .for_each(|(dynamic_entity, (transform, velocity, shape, mut hits))| {
            'horizontal: {
                let horizontal_movement = glam::vec3(velocity.0.x, 0., velocity.0.z) * delta_time;

                if horizontal_movement == glam::Vec3::ZERO {
                    break 'horizontal;
                }

                transform.translation += horizontal_movement;

                let prepped = PreppedCollisionShape::from_collision_shape(
                    shape,
                    transform.translation,
                    transform.scale,
                );

                let collisions = check_static_against(
                    state.world(),
                    dynamic_entity,
                    &prepped,
                    CollisionDirection::Horizontal,
                );
                if !collisions.is_empty() {
                    transform.translation -= horizontal_movement;
                    velocity.0.x *= 0.4;
                    velocity.0.z *= 0.4;

                    if let Some(hits) = &mut hits {
                        collisions.into_iter().for_each(|entity| {
                            hits.hits.push((entity, CollisionDirection::Horizontal))
                        });
                    }
                }
            }

            'vertical: {
                let vertical_movement = velocity.0.y * delta_time;

                if vertical_movement == 0. {
                    break 'vertical;
                }

                transform.translation.y += vertical_movement;

                let prepped = PreppedCollisionShape::from_collision_shape(
                    shape,
                    transform.translation,
                    transform.scale,
                );

                let (static_direction, dynamic_direction) = match vertical_movement >= 0. {
                    true => (
                        CollisionDirection::VerticalBottom,
                        CollisionDirection::VerticalTop,
                    ),
                    false => (
                        CollisionDirection::VerticalTop,
                        CollisionDirection::VerticalBottom,
                    ),
                };

                let collisions =
                    check_static_against(state.world(), dynamic_entity, &prepped, static_direction);
                if !collisions.is_empty() {
                    transform.translation.y -= vertical_movement;
                    velocity.0.y *= 0.4;

                    if let Some(hits) = &mut hits {
                        collisions
                            .into_iter()
                            .for_each(|entity| hits.hits.push((entity, dynamic_direction)));
                    }
                }

                if let Some(hits) = &hits {
                    if !hits.hits.is_empty() {}
                }
            }
        });
}

fn check_static_against(
    world: &World,
    other: Entity,
    prepped: &PreppedCollisionShape,
    direction: CollisionDirection,
) -> Vec<Entity> {
    world
        .query::<(&PreppedCollisionShape, Option<&mut CollisionHits>)>()
        .with::<&StaticCollisionType>()
        .without::<(&DynamicCollisionType, &TriggerCollisionType)>()
        .into_iter()
        .filter_map(|(entity, (static_collision, mut hits))| {
            match prepped.check_collision(static_collision) {
                true => {
                    if let Some(hits) = &mut hits {
                        hits.hits.push((other, direction));
                    }
                    Some(entity)
                }
                false => None,
            }
        })
        .collect()
}

fn update_grounded(state: &mut State) {
    let remove_grounded = state
        .world_mut()
        .query_mut::<&CollisionHits>()
        .with::<(&Gravity, &Grounded)>()
        .into_iter()
        .filter_map(|(entity, hits)| {
            match hits
                .hits
                .iter()
                .any(|(_, collision)| *collision == CollisionDirection::VerticalBottom)
            {
                true => None,
                false => Some(entity),
            }
        })
        .collect::<Vec<_>>();

    let add_grounded = state
        .world_mut()
        .query_mut::<&CollisionHits>()
        .with::<&Gravity>()
        .without::<&Grounded>()
        .into_iter()
        .filter_map(|(entity, hits)| {
            match hits
                .hits
                .iter()
                .any(|(_, collision)| *collision == CollisionDirection::VerticalBottom)
            {
                true => Some(entity),
                false => None,
            }
        })
        .collect::<Vec<_>>();

    remove_grounded.into_iter().for_each(|entity| {
        state.world_mut().remove_one::<Grounded>(entity).unwrap();
    });

    add_grounded.into_iter().for_each(|entity| {
        state.world_mut().insert_one(entity, Grounded).unwrap();
    });
}

#[inline]
fn clear_controller_actions(state: &mut State) {
    state
        .world_mut()
        .query_mut::<&mut CharacterController>()
        .into_iter()
        .for_each(|(_, controller)| controller.movement_action_queue.clear());
}

//====================================================================

enum PreppedCollisionShape {
    Box((Range, Range, Range)),
}

struct Range {
    min: f32,
    max: f32,
}

impl PreppedCollisionShape {
    fn from_collision_shape(
        value: &CollisionShape,
        translation: glam::Vec3,
        scale: glam::Vec3,
    ) -> Self {
        match value {
            CollisionShape::Box {
                half_width,
                half_height,
                half_depth,
            } => PreppedCollisionShape::Box((
                Range {
                    min: translation.x - half_width * scale.x,
                    max: translation.x + half_width * scale.x,
                },
                Range {
                    min: translation.y - half_height * scale.y,
                    max: translation.y + half_height * scale.y,
                },
                Range {
                    min: translation.z - half_depth * scale.z,
                    max: translation.z + half_depth * scale.z,
                },
            )),
        }
    }

    fn check_collision(&self, other: &PreppedCollisionShape) -> bool {
        match (self, other) {
            (PreppedCollisionShape::Box(a), PreppedCollisionShape::Box(b)) => aabb_vs_aabb(a, b),
        }
    }
}

#[inline]
fn aabb_vs_aabb(a: &(Range, Range, Range), b: &(Range, Range, Range)) -> bool {
    a.0.min <= b.0.max
        && a.0.max >= b.0.min
        && a.1.min <= b.1.max
        && a.1.max >= b.1.min
        && a.2.min <= b.2.max
        && a.2.max >= b.2.min
}

//====================================================================
