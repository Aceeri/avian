#[cfg(feature = "2d")]
pub extern crate parry2d_f64 as parry;

#[cfg(feature = "3d")]
pub extern crate parry3d_f64 as parry;

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

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy::math::{DVec2, DVec3, DMat3, DQuat};
use bevy_prototype_debug_lines::*;
use parry::math::Isometry;
use prelude::*;

#[cfg(feature = "2d")]
pub type Vector = DVec2;

#[cfg(feature = "3d")]
pub type Vector = DVec3;

pub const DELTA_TIME: f64 = 1.0 / 60.0;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

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

        // Add stages
        app.add_stage_before(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel().with_run_criteria(run_criteria),
        );

        // Add plugins for physics simulation loop
        app.add_plugin(PreparePlugin)
            .add_plugin(BroadPhasePlugin)
            .add_plugin(IntegratorPlugin)
            .add_plugin(SolverPlugin);

        app.add_plugin(DebugLinesPlugin::default());

        #[cfg(feature = "debug-render-aabbs")]
        app.add_system(draw_aabbs);
    }
}

#[cfg(feature = "debug-render-aabbs")]
fn draw_aabbs(aabbs: Query<&ColliderAabb>, mut lines: ResMut<DebugLines>) {
    #[cfg(feature = "2d")]
    for aabb in aabbs.iter() {
        let v1 = DVec3::new(aabb.mins.x, aabb.mins.y, 0.0);
        let v2 = DVec3::new(aabb.maxs.x, aabb.mins.y, 0.0);
        let v3 = DVec3::new(aabb.maxs.x, aabb.maxs.y, 0.0);
        let v4 = DVec3::new(aabb.mins.x, aabb.maxs.y, 0.0);

        lines.line(v1, v2, 0.0);
        lines.line(v2, v3, 0.0);
        lines.line(v3, v4, 0.0);
        lines.line(v4, v1, 0.0);
    }

    #[cfg(feature = "3d")]
    for aabb in aabbs.iter() {
        let v1 = DVec3::new(aabb.mins.x, aabb.mins.y, aabb.mins.z);
        let v2 = DVec3::new(aabb.maxs.x, aabb.mins.y, aabb.mins.z);
        let v3 = DVec3::new(aabb.maxs.x, aabb.maxs.y, aabb.mins.z);
        let v4 = DVec3::new(aabb.mins.x, aabb.maxs.y, aabb.mins.z);
        let v5 = DVec3::new(aabb.mins.x, aabb.mins.y, aabb.maxs.z);
        let v6 = DVec3::new(aabb.maxs.x, aabb.mins.y, aabb.maxs.z);
        let v7 = DVec3::new(aabb.maxs.x, aabb.maxs.y, aabb.maxs.z);
        let v8 = DVec3::new(aabb.mins.x, aabb.maxs.y, aabb.maxs.z);

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

#[derive(Resource, Debug, Default)]
pub struct XpbdLoop {
    pub(crate) has_added_time: bool,
    pub(crate) accumulator: f64,
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

fn run_criteria(
    time: Res<Time>,
    substeps: Res<NumSubsteps>,
    mut state: ResMut<XpbdLoop>,
) -> ShouldRun {
    if state.paused && state.queued_steps == 0 {
        return ShouldRun::No;
    }

    if !state.has_added_time {
        state.has_added_time = true;

        if state.paused {
            state.accumulator += DELTA_TIME * state.queued_steps as f64;
        } else {
            state.accumulator += time.delta_seconds_f64();
        }
    }

    if state.substepping {
        state.current_substep += 1;

        if state.current_substep < substeps.0 {
            return ShouldRun::YesAndCheckAgain;
        } else {
            // We finished a whole step
            if state.paused && state.queued_steps > 0 {
                state.queued_steps -= 1;
            } else {
                state.accumulator -= DELTA_TIME;
            }

            state.current_substep = 0;
            state.substepping = false;
        }
    }

    if state.accumulator >= DELTA_TIME {
        state.substepping = true;
        state.current_substep = 0;
        ShouldRun::YesAndCheckAgain
    } else {
        state.has_added_time = false;
        ShouldRun::No
    }
}

fn first_substep(state: Res<XpbdLoop>) -> ShouldRun {
    if state.current_substep == 0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
