#[cfg(feature = "2d")]
pub extern crate parry2d as parry;

#[cfg(feature = "3d")]
pub extern crate parry3d as parry;

pub mod bundles;
pub mod collision;
pub mod components;
pub mod constraints;
pub mod resources;
pub mod steps;

pub mod prelude {
    pub use crate::{
        bundles::*,
        components::*,
        constraints::{joints::*, *},
        resources::*,
        steps::*,
        *,
    };
}

mod utils;

use bevy::prelude::*;
//use bevy_prototype_debug_lines::*;
use parry::math::Isometry;
use prelude::*;

#[cfg(feature = "2d")]
pub type Vector = Vec2;

#[cfg(feature = "3d")]
pub type Vector = Vec3;

pub const DELTA_TIME: f32 = 1.0 / 60.0;

pub struct XpbdPlugin;

impl Plugin for XpbdPlugin {
    fn build(&self, app: &mut App) {
        // Init resources and register component types
        app.init_resource::<NumSubsteps>()
            .init_resource::<NumPosIters>()
            .init_resource::<SubDeltaTime>()
            .init_resource::<XpbdLoop>()
            .init_resource::<Gravity>()
            .register_type::<RigidBody>()
            .register_type::<Pos>()
            .register_type::<Rot>()
            .register_type::<PrevPos>()
            .register_type::<PrevRot>()
            .register_type::<LinVel>()
            .register_type::<AngVel>()
            .register_type::<PreSolveLinVel>()
            .register_type::<PreSolveAngVel>()
            .register_type::<Restitution>()
            .register_type::<Friction>()
            .register_type::<ExternalForce>()
            .register_type::<ExternalTorque>()
            .register_type::<Mass>()
            .register_type::<InvMass>()
            .register_type::<Inertia>()
            .register_type::<InvInertia>()
            .register_type::<LocalCom>();

        // Add plugins for physics simulation loop
        app.add_plugin(PreparePlugin)
            .add_plugin(BroadPhasePlugin)
            .add_plugin(IntegratorPlugin)
            .add_plugin(SolverPlugin);

        //app.add_plugin(DebugLinesPlugin::default());

        #[cfg(feature = "debug-render-aabbs")]
        app.add_system(draw_aabbs);
    }
}

/*
#[cfg(feature = "debug-render-aabbs")]
fn draw_aabbs(aabbs: Query<&ColliderAabb>, mut lines: ResMut<DebugLines>) {
    #[cfg(feature = "2d")]
    for aabb in aabbs.iter() {
        let v1 = Vec3::new(aabb.mins.x, aabb.mins.y, 0.0);
        let v2 = Vec3::new(aabb.maxs.x, aabb.mins.y, 0.0);
        let v3 = Vec3::new(aabb.maxs.x, aabb.maxs.y, 0.0);
        let v4 = Vec3::new(aabb.mins.x, aabb.maxs.y, 0.0);

        lines.line(v1, v2, 0.0);
        lines.line(v2, v3, 0.0);
        lines.line(v3, v4, 0.0);
        lines.line(v4, v1, 0.0);
    }

    #[cfg(feature = "3d")]
    for aabb in aabbs.iter() {
        let v1 = Vec3::new(aabb.mins.x, aabb.mins.y, aabb.mins.z);
        let v2 = Vec3::new(aabb.maxs.x, aabb.mins.y, aabb.mins.z);
        let v3 = Vec3::new(aabb.maxs.x, aabb.maxs.y, aabb.mins.z);
        let v4 = Vec3::new(aabb.mins.x, aabb.maxs.y, aabb.mins.z);
        let v5 = Vec3::new(aabb.mins.x, aabb.mins.y, aabb.maxs.z);
        let v6 = Vec3::new(aabb.maxs.x, aabb.mins.y, aabb.maxs.z);
        let v7 = Vec3::new(aabb.maxs.x, aabb.maxs.y, aabb.maxs.z);
        let v8 = Vec3::new(aabb.mins.x, aabb.maxs.y, aabb.maxs.z);

        lines.line(v1, v2, 0.0);
        lines.line(v2, v3, 0.0);
        lines.line(v3, v4, 0.0);
        lines.line(v4, v1, 0.0);
        lines.line(v5, v6, 0.0);
        lines.line(v6, v7, 0.0);
        lines.line(v7, v8, 0.0);
        lines.line(v8, v5, 0.0);
        lines.line(v1, v5, 0.0);
        lines.line(v2, v6, 0.0);
        lines.line(v3, v7, 0.0);
        lines.line(v4, v8, 0.0);
    }
}
 */
#[derive(Resource, Debug, Default)]
pub struct XpbdLoop {
    pub(crate) substepping: bool,
    pub(crate) current_substep: u32,
    pub(crate) queued_steps: u32,
    pub paused: bool,
}

impl XpbdLoop {
    pub fn step(&mut self) {
        self.queued_steps += 1;
    }
    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn resume(&mut self) {
        self.paused = false;
    }
}

pub fn pause(mut xpbd_loop: ResMut<XpbdLoop>) {
    xpbd_loop.pause();
}

pub fn resume(mut xpbd_loop: ResMut<XpbdLoop>) {
    xpbd_loop.resume();
}

fn first_substep(state: Res<XpbdLoop>) -> bool {
    state.current_substep == 0
}
