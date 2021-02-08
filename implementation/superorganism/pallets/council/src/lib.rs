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
use frame_support::{decl_module, decl_storage, dispatch::{DispatchError, Vec}};
use frame_system::{ensure_signed, ensure_root};
use pallet_community_identity::{ProofType, IdentityId, IdentityLevel, traits::PeerReviewedPhysicalIdentity};
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
/// Public interface to Council
pub mod traits;

pub type Ticket = u64;
pub type BlockNumber<T> = <T as frame_system::Trait>::BlockNumber;
// TODO: Change from Vec<u8> to fixed length type
pub type DocumentCID = Vec<u8>;


/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	// type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
	/// Define Identity type. Must implement PeerReviewedPhysicalIdentity trait
	type Identity: PeerReviewedPhysicalIdentity<ProofType, IdentityId = IdentityId<Self>,
						IdentityLevel = IdentityLevel, Address = Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Council {
		pub TicketNumber get(fn ticket): Ticket = 0;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// As an identified user, vote for a council member
		#[weight = 10_000]
		fn vote_council_member(origin, candidate: IdentityId<T>) {
			let caller = ensure_signed(origin)?;
			Self::do_vote_council_member(T::Identity::get_identity_id(&caller), candidate)?;
		}

		/// As an identified user, vote to reelect council
		#[weight = 10_000]
		fn vote_reelect_council(origin) {
			let caller = ensure_signed(origin)?;
			Self::do_vote_reelect_council(T::Identity::get_identity_id(&caller))?;
		}

		/// As an identified user, vote to reelect a specific council member
		#[weight = 10_000]
		fn vote_reelect(origin, member: IdentityId<T>) {
			let caller = ensure_signed(origin)?;
			Self::do_vote_reelect(T::Identity::get_identity_id(&caller), member)?;
		}

		/// As root, queue a poll
		#[weight = 10_000]
		fn add_poll(origin, documents: Vec<DocumentCID>, until: BlockNumber<T>) {
			ensure_root(origin)?;
			Self::do_add_poll(documents, until)?;
		}

		/// As a council member, vote for a poll
		#[weight = 10_000]
		fn vote_poll(origin, poll: Ticket, accept: bool) {
			let caller = ensure_signed(origin)?;
			Self::do_vote_poll(T::Identity::get_identity_id(&caller), poll, accept)?;
		}
	}
}

impl<T: Trait> Module<T> {
	fn do_vote_council_member(_voter: IdentityId<T>, _candidate: IdentityId<T>)
		-> Result<(), DispatchError>
	{
		// TODO implement
		Ok(())
	}

	fn do_vote_reelect_council(_voter: IdentityId<T>) -> Result<(), DispatchError> {
		// TODO implement
		Ok(())
	}

	fn do_vote_reelect(_voter: IdentityId<T>, _member: IdentityId<T>) -> Result<(), DispatchError> {
		// TODO implement
		Ok(())
	}

	fn do_add_poll(_document: Vec<DocumentCID>, _until: BlockNumber<T>) -> Result<Ticket, DispatchError> {
		let ticket: Ticket = <TicketNumber>::get();
		TicketNumber::put(ticket + 1);
		Ok(ticket)
	}

	fn do_vote_poll(_member: IdentityId<T>, _poll: Ticket, _accept: bool) -> Result<(), DispatchError> {
		Ok(())
	}

	fn do_get_result(_poll: &Ticket) -> Option<Vec<(IdentityId<T>, bool)>> {
		Some(Vec::from([(Default::default(), true), (Default::default(), true), (Default::default(), true),
			(Default::default(), true), (Default::default(), true), (Default::default(), true)]))
	}
}


impl<T: Trait> traits::Council for Module<T>
{
	type IdentityId = IdentityId<T>;
	type Ticket = Ticket;
	type BlockNumber = BlockNumber<T>;
	type DocumentCID = DocumentCID;

	/// As an identified user, vote for a council member
	fn vote_council_member(voter: Self::IdentityId, candidate: Self::IdentityId)
		-> Result<(), DispatchError>
	{
		Self::do_vote_council_member(voter, candidate)
	}

	/// As an identified user, vote to reelect council
	fn vote_reelect_council(voter: Self::IdentityId) -> Result<(), DispatchError> {
		Self::do_vote_reelect_council(voter)
	}

	/// As an identified user, vote to reelect a specific council member
	fn vote_reelect(voter: Self::IdentityId, member: Self::IdentityId)
		-> Result<(), DispatchError> 
	{
		Self::do_vote_reelect(voter, member)
	}

	/// As root, queue a poll
	fn add_poll(documents: Vec<Self::DocumentCID>, until: Self::BlockNumber) 
		-> Result<Self::Ticket, DispatchError> 
	{
		Self::do_add_poll(documents, until)
	}

	/// As a council member, vote for a poll
	fn vote_poll(member: Self::IdentityId, poll: Self::Ticket, accept: bool)
		-> Result<(), DispatchError>
	{
		Self::do_vote_poll(member, poll, accept)
	}

	/// Retrieve result of a poll
	fn get_result(poll: &Self::Ticket) -> Option<Vec<(Self::IdentityId, bool)>> {
		Self::do_get_result(poll)
	}
}
