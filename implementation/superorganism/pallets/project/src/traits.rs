// Copyright 2020 Harald Heckmann

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use frame_support::dispatch::{Codec, DispatchError, EncodeLike, fmt::Debug, Vec};
use crate::types::{DocumentCID, ProjectID};

/// Beginning of the project trait definition.
/// A project is spawned from a proposal and the concerns.
/// Project leaders can apply for that project for a specific time interval.
/// Verified identities can vote for a project leader.
/// An elected project leader creates open positions.
/// Workers can apply for that open positions.
/// The project leader selects applicants for the open roles.
/// The project is started, funds are allocated (based on the estimation in the application of the PL)
/// and every worker in the project is paid after a specific time interval.
/// The project leader can fire workers anytime (giving a grace period).
/// The workers can vote to fire the project leader. The project is halted and applications
/// for a new project leader is opened again.
pub trait ProjectTrait
{
	type Balance: Codec + Clone + Debug + Eq + PartialEq;
	type IdentityId: Codec + Clone + Eq + EncodeLike + Debug;
	type ProposalWinner: Codec + Clone + Eq + Debug + PartialEq;
	type Project: Codec + Clone + Debug + Eq + PartialEq;

	/// As root, spawn a project from a proposal
	fn spawn_project(proposal: Self::ProposalWinner) -> Result<Self::Project, DispatchError>;
	/// As an identified user, apply as project leader
	fn application_project_leader(who: Self::IdentityId, project: ProjectID, application: DocumentCID)
		-> Result<(), DispatchError>;
	/// As an identified user, Vote for project leader
	fn vote_project_leader(voter: Self::IdentityId, pl: Self::IdentityId, project: ProjectID)
		-> Result<(), DispatchError>;
	/// As a project leader, open positions
	fn open_position(pl: Self::IdentityId, project: ProjectID, position: DocumentCID)
		-> Result<(), DispatchError>;
	/// As an identified user, apply for a position
	fn apply(applicant: Self::IdentityId, project: ProjectID, position: DocumentCID, application: DocumentCID)
		-> Result<(), DispatchError>;
	// As a project leader, accept application and offer salary
	fn offer_applicant(pl: Self::IdentityId, applicant: Self::IdentityId, project: ProjectID,
		position: DocumentCID, application: DocumentCID, salary: Self::Balance) -> Result<(), DispatchError>;
	// As an applicant, accept an offer	
	fn accept_offer(applicant: Self::IdentityId, project: ProjectID, position: DocumentCID, salary: Self::Balance)
		-> Result<(), DispatchError>;
	// As a participant, vote to replace a colleague
	fn vote_replace(pl: Self::IdentityId, worker: Self::IdentityId, project: ProjectID)
		-> Result<(), DispatchError>;
	/// Get project
	fn get_project(project: ProjectID) -> Result<Option<Self::Project>, DispatchError>;
	/// Get all projects
	fn get_projects() -> Result<Vec<Self::Project>, DispatchError>;
}
