mod fixed;
mod prismatic;
mod revolute;
mod spherical;

pub use fixed::*;
pub use prismatic::*;
pub use revolute::*;
pub use spherical::*;

use crate::prelude::*;
use bevy::prelude::*;
use std::f64::consts::PI;

pub trait Joint: Component + PositionConstraint + AngularConstraint {
    fn new_with_compliance(entity1: Entity, entity2: Entity, compliance: f64) -> Self;

    fn with_local_anchor_1(self, anchor: Vector) -> Self;

    fn with_local_anchor_2(self, anchor: Vector) -> Self;

    fn with_lin_vel_damping(self, damping: f64) -> Self;

    fn with_ang_vel_damping(self, damping: f64) -> Self;

    fn entities(&self) -> [Entity; 2];

    fn damping_lin(&self) -> f64;

    fn damping_ang(&self) -> f64;

    fn constrain(
        &mut self,
        body1: &mut RigidBodyQueryItem,
        body2: &mut RigidBodyQueryItem,
        sub_dt: f64,
    );

    #[allow(clippy::too_many_arguments)]
    fn limit_distance(
        &mut self,
        min: f64,
        max: f64,
        r1: Vector,
        r2: Vector,
        pos1: &Pos,
        pos2: &Pos,
    ) -> Vector {
        let pos_offset = (pos2.0 + r2) - (pos1.0 + r1);
        let distance = pos_offset.length();

        if distance <= f64::EPSILON {
            return Vector::ZERO;
        }

        // Equation 25
        if distance < min {
            // Separation distance lower limit
            -pos_offset / distance * (distance - min)
        } else if distance > max {
            // Separation distance upper limit
            -pos_offset / distance * (distance - max)
        } else {
            Vector::ZERO
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn limit_distance_along_axis(
        &mut self,
        min: f64,
        max: f64,
        axis: Vector,
        r1: Vector,
        r2: Vector,
        pos1: &Pos,
        pos2: &Pos,
    ) -> Vector {
        let pos_offset = (pos2.0 + r2) - (pos1.0 + r1);
        let a = pos_offset.dot(axis);

        // Equation 25
        if a < min {
            // Separation distance lower limit
            -axis * (a - min)
        } else if a > max {
            // Separation distance upper limit
            -axis * (a - max)
        } else {
            Vector::ZERO
        }
    }

    fn limit_angle(
        n: DVec3,
        n1: DVec3,
        n2: DVec3,
        alpha: f64,
        beta: f64,
        max_correction: f64,
    ) -> Option<DVec3> {
        let mut phi = n1.cross(n2).dot(n).asin();

        if n1.dot(n2) < 0.0 {
            phi = PI - phi;
        }

        if phi > PI {
            phi -= 2.0 * PI;
        }

        if phi < -PI {
            phi += 2.0 * PI;
        }

        if phi < alpha || phi > beta {
            phi = phi.clamp(alpha, beta);

            let rot = DQuat::from_axis_angle(n, phi);
            let mut omega = rot.mul_vec3(n1).cross(n2);

            phi = omega.length();

            if phi > max_correction {
                omega *= max_correction / phi;
            }

            return Some(omega);
        }

        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointLimit {
    pub min: f64,
    pub max: f64,
}

impl JointLimit {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}
