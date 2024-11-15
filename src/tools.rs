//====================================================================

use hecs::Entity;
use hecs_engine::{
    common::{GlobalTransform, Transform},
    engine::State,
};

//====================================================================

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
