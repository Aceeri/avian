use crate::{components::*, Vector};

use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct RigidBodyBundle {
    pub rigid_body: RigidBody,

    #[bundle]
    pub transform: TransformBundle,

    pub previous_transform: PrevTransform,

    pub velocity: Velocity,
    pub presolve_velocity: PreSolveVelocity,

    pub external_force: ExternalForce,
    pub external_torque: ExternalTorque,
    pub restitution: Restitution,
    pub friction: Friction,

    pub mass: Mass,
    pub inertia: Inertia,
    pub local_center_of_mass: LocalCom,
}

impl RigidBodyBundle {
    pub fn new_dynamic() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            ..default()
        }
    }

    pub fn new_static() -> Self {
        Self {
            rigid_body: RigidBody::Static,
            ..default()
        }
    }

    pub fn new_kinematic() -> Self {
        Self {
            rigid_body: RigidBody::Kinematic,
            ..default()
        }
    }

    /// Computes the mass properties that a [`Collider`] would have with a given density, and adds those to the body.
    pub fn with_mass_props_from_shape(self, shape: &Shape, density: f32) -> Self {
        let ColliderMassProperties {
            mass,
            inertia,
            local_center_of_mass,
            ..
        } = ColliderMassProperties::from_shape_and_density(shape, density);

        Self {
            mass,
            inertia,
            local_center_of_mass,
            ..self
        }
    }
}

#[derive(Bundle, Default)]
pub struct ColliderBundle {
    collider_shape: ColliderShape,
    collider_aabb: ColliderAabb,
    pub mass_props: ColliderMassProperties,
    pub(crate) prev_mass_props: PrevColliderMassProperties,
}

impl ColliderBundle {
    /// Creates a new [`ColliderBundle`] from a given [`ColliderShape`] and density.
    pub fn new(shape: &Shape, density: f32) -> Self {
        let aabb = ColliderAabb::from_shape(shape);
        let mass_props = ColliderMassProperties::from_shape_and_density(shape, density);

        Self {
            collider_shape: ColliderShape(shape.to_owned()),
            collider_aabb: aabb,
            mass_props,
            prev_mass_props: PrevColliderMassProperties(ColliderMassProperties::ZERO),
        }
    }

    pub fn update_mass_props(&mut self) {
        self.mass_props = ColliderMassProperties::from_shape_and_density(
            &self.collider_shape.0,
            self.mass_props.density,
        );
    }
}
