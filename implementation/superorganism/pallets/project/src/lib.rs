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

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_event, decl_module, decl_storage, dispatch::{DispatchError, Vec},
					traits::{Currency, ReservableCurrency}};
use frame_system::{ensure_signed, ensure_root};
use pallet_community_identity::{IdentityId, IdentityLevel, ProofType, traits::PeerReviewedPhysicalIdentity};
use pallet_proposal_types::ProposalWinner;
use crate::{traits::ProjectTrait, types::{DocumentCID, Project, ProjectID}};
pub mod traits;
pub mod types;

// use frame_system::ensure_root;
// use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Type that manages balances
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Define Identity type. Must implement PeerReviewedPhysicalIdentity trait
	type Identity: PeerReviewedPhysicalIdentity<ProofType, IdentityId = IdentityId<Self>,
						IdentityLevel = IdentityLevel, Address = Self::AccountId>;
}

decl_event! {
	pub enum Event<T> where PRJ = Project<BalanceOf<T>, <T as frame_system::Trait>::BlockNumber, IdentityId<T>> {
		/// A new project has been spawned \[Project\]
		ProjectSpawned(PRJ),
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as ProjectPallet {
		pub ProjectNumber get(fn project_number): ProjectID = 0;
		pub ProjectStorage get(fn project): map hasher(identity)
			ProjectID => Option<Project<BalanceOf<T>, T::BlockNumber, IdentityId<T>>> = None;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// As root, spawn a project from a proposal
		#[weight = 10_000]
		fn spawn_project(origin, proposal: ProposalWinner<IdentityId<T>>) {
			ensure_root(origin)?;
			Self::do_spawn_project(proposal)?;
		}

		/// As an identified user, apply as project leader
		#[weight = 10_000]
		fn application_project_leader(origin, project: ProjectID, application: DocumentCID) {
			let caller = ensure_signed(origin)?;
			Self::do_application_project_leader(T::Identity::get_identity_id(&caller), project, application)?;
		}

		/// As an identified user, Vote for project leader
		#[weight = 10_000]
		fn vote_project_leader(origin, pl: IdentityId<T>, project: ProjectID) {
			let caller = ensure_signed(origin)?;
			Self::do_vote_project_leader(T::Identity::get_identity_id(&caller), pl, project)?;
		}

		/// As a project leader, open positions
		#[weight = 10_000]
		fn open_position(origin, project: ProjectID, position: DocumentCID) {
			let caller = ensure_signed(origin)?;
			Self::do_open_position(T::Identity::get_identity_id(&caller), project, position)?;
		}

		/// As an identified user, apply for a position
		#[weight = 10_000]
		fn apply(origin, project: ProjectID, position: DocumentCID, application: DocumentCID) {
			let caller = ensure_signed(origin)?;
			Self::do_apply(T::Identity::get_identity_id(&caller), project, position, application)?;
		}

		/// As a project leader, accept application and offer salary
		#[weight = 10_000]
		fn offer_applicant(origin, applicant: IdentityId<T>, project: ProjectID,
			position: DocumentCID, application: DocumentCID, salary: BalanceOf<T>)
		{
			let caller = ensure_signed(origin)?;
			Self::do_offer_applicant(T::Identity::get_identity_id(&caller), applicant,
				project, position, application, salary)?;
		}

		/// As an applicant, accept an offer	
		#[weight = 10_000]
		fn accept_offer(origin, project: ProjectID, position: DocumentCID, salary: BalanceOf<T>) {
			let caller = ensure_signed(origin)?;
			Self::do_accept_offer(T::Identity::get_identity_id(&caller), project, position, salary)?;
		}

		/// As a participant, vote to replace a colleague
		#[weight = 10_000]
		fn vote_replace(origin, colleague: IdentityId<T>, project: ProjectID) {
			let caller = ensure_signed(origin)?;
			Self::do_vote_replace(colleague, T::Identity::get_identity_id(&caller), project)?;
		}
	}
}

impl<T: Trait> Module<T> {
	/// As root, spawn a project from a proposal
	fn do_spawn_project(proposal: ProposalWinner<IdentityId<T>>) 
		-> Result<Project<BalanceOf<T>, T::BlockNumber, IdentityId<T>>, DispatchError>
	{
		let pn: ProjectID = <ProjectNumber>::get();
		let project = Project::new(pn, proposal);
		ProjectStorage::<T>::insert(pn, &project);
		ProjectNumber::put(pn+1);
		Self::deposit_event(Event::<T>::ProjectSpawned(project.clone()));
		Ok(project)
	}

	/// As an identified user, apply as project leader
	fn do_application_project_leader(_who: IdentityId<T>, _project: ProjectID, _application: DocumentCID)
		-> Result<(), DispatchError>
	{
		Ok(())
	}

	/// As an identified user, Vote for project leader
	fn do_vote_project_leader(_voter: IdentityId<T>, _pl: IdentityId<T>, _project: ProjectID)
		-> Result<(), DispatchError>
	{
		Ok(())
	}

	/// As a project leader, open positions
	fn do_open_position(_pl: IdentityId<T>, _project: ProjectID, _position: DocumentCID)
		-> Result<(), DispatchError>
	{
		Ok(())
	}

	/// As an identified user, apply for a position
	fn do_apply(_applicant: IdentityId<T>, _project: ProjectID, _position: DocumentCID, _application: DocumentCID)
		-> Result<(), DispatchError>
	{
		Ok(())
	}

	/// As a project leader, accept application and offer salary
	fn do_offer_applicant(_pl: IdentityId<T>, _applicant: IdentityId<T>, _project: ProjectID,
		_position: DocumentCID, _application: DocumentCID, _salary: BalanceOf<T>) -> Result<(), DispatchError>
	{
		Ok(())
	}

	/// As an applicant, accept an offer	
	fn do_accept_offer(_applicant: IdentityId<T>, _project: ProjectID, _position: DocumentCID, _salary: BalanceOf<T>)
		-> Result<(), DispatchError>
	{
		Ok(())
	}

	/// As a participant, vote to replace a colleague
	fn do_vote_replace(_colleague: IdentityId<T>, _worker: IdentityId<T>, _project: ProjectID)
		-> Result<(), DispatchError>
	{
		Ok(())
	}

	/// Get project
	fn do_get_project(project: ProjectID) -> 
		Result<Option<Project<BalanceOf<T>, T::BlockNumber, IdentityId<T>>>, DispatchError>
	{
		Ok(<ProjectStorage<T>>::get(project))
	}

	/// Get all projects
	fn do_get_projects() -> Result<Vec<Project<BalanceOf<T>, T::BlockNumber, IdentityId<T>>>, DispatchError> {
		let mut result: Vec<Project<BalanceOf<T>, T::BlockNumber, IdentityId<T>>> = Vec::new();

		for (_, project) in <ProjectStorage<T>>::iter() {
			result.push(project);
		}

		Ok(result)
	}
}

impl<T: Trait> ProjectTrait for Module<T> {
	type Balance = BalanceOf<T>;
	type IdentityId = IdentityId<T>;
	type ProposalWinner = ProposalWinner<IdentityId<T>>;
	type Project = Project<BalanceOf<T>, T::BlockNumber, IdentityId<T>>;

	/// As root, spawn a project from a proposal
	fn spawn_project(proposal: Self::ProposalWinner) -> Result<Self::Project, DispatchError> {
		Self::do_spawn_project(proposal)
	}

	/// As an identified user, apply as project leader
	fn application_project_leader(who: Self::IdentityId, project: ProjectID, application: DocumentCID)
		-> Result<(), DispatchError>
	{
		Self::do_application_project_leader(who, project, application)
	}

	/// As an identified user, Vote for project leader
	fn vote_project_leader(voter: Self::IdentityId, pl: Self::IdentityId, project: ProjectID)
		-> Result<(), DispatchError>
	{
		Self::do_vote_project_leader(voter, pl, project)
	}

	/// As a project leader, open positions
	fn open_position(pl: Self::IdentityId, project: ProjectID, position: DocumentCID)
		-> Result<(), DispatchError>
	{
		Self::do_open_position(pl, project, position)
	}

	/// As an identified user, apply for a position
	fn apply(applicant: Self::IdentityId, project: ProjectID, position: DocumentCID, application: DocumentCID)
		-> Result<(), DispatchError>
	{
		Self::do_apply(applicant, project, position, application)
	}

	/// As a project leader, accept application and offer salary
	fn offer_applicant(pl: Self::IdentityId, applicant: Self::IdentityId, project: ProjectID,
		position: DocumentCID, application: DocumentCID, salary: BalanceOf<T>) -> Result<(), DispatchError>
	{
		Self::do_offer_applicant(pl, applicant, project, position, application, salary)
	}

	/// As an applicant, accept an offer	
	fn accept_offer(applicant: Self::IdentityId, project: ProjectID, position: DocumentCID, salary: BalanceOf<T>)
		-> Result<(), DispatchError>
	{
		Self::do_accept_offer(applicant, project, position, salary)
	}

	/// As a participant, vote to replace a colleague
	fn vote_replace(colleague: Self::IdentityId, worker: Self::IdentityId, project: ProjectID)
		-> Result<(), DispatchError>
	{
		Self::do_vote_replace(colleague, worker, project)
	}

	/// Get project
	fn get_project(project: ProjectID) -> Result<Option<Self::Project>, DispatchError> {
		Self::do_get_project(project)
	}

	/// Get all projects
	fn get_projects() -> Result<Vec<Self::Project>, DispatchError> {
		Self::do_get_projects()
	}
}
