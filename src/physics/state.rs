// Copyright (C) 2023 Lily Lyons
//
// This file is part of wormhole.
//
// wormhole is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// wormhole is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with wormhole.  If not, see <http://www.gnu.org/licenses/>.

use bevy_ecs::prelude::*;
use rapier3d::prelude::*;

#[derive(Resource)]
pub struct State {
    pub physics_pipeline: PhysicsPipeline,
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impluse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
}

impl State {
    pub fn new() -> Self {
        let physics_pipeline = PhysicsPipeline::new();
        let gravity = vector![0.0, 9.81, 0.0];
        let integration_parameters = IntegrationParameters::default();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();
        let impluse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();

        Self {
            physics_pipeline,
            gravity,
            integration_parameters,
            island_manager,
            broad_phase,
            narrow_phase,
            rigid_body_set,
            collider_set,
            impluse_joint_set,
            multibody_joint_set,
            ccd_solver,
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impluse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            // todo dispatch to ecs?
            &(),
            &(),
        );
    }
}
