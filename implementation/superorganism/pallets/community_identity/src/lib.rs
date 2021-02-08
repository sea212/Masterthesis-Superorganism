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
use frame_support::{
	decl_module,
	dispatch::{DispatchError, fmt::Debug, Vec},
	Parameter,
	sp_runtime::traits::{AtLeast32Bit, Scale},
};
use frame_system::ensure_signed;
use codec::{Codec, Decode, Encode, EncodeLike};
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
/// Public interface to PhysicalIdentity
pub mod traits;


pub type IdentityLevel = u8;
pub type ProofType = [u8; 32];
pub type IdentityId<T> = <T as frame_system::Trait>::AccountId;
type Ticket<T> = <T as frame_system::Trait>::AccountId;

/// Structure that contains the proof
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PhysicalProof<Timestamp, ProofData> where
	ProofData: Codec + Clone + Debug + Eq + PartialEq,
	Timestamp: AtLeast32Bit + Parameter + Default + Debug + Copy,
{
	proof: ProofData,
	date: Timestamp,
}

/// Structure that contains the identity ID, level and proof
#[derive(Clone, Decode, Debug, Encode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PhysicalIdentityData<Timestamp, AccountId, ProofData> where
	ProofData: Codec + Clone + Debug + Eq + PartialEq,
	AccountId: Codec + Clone + Debug + EncodeLike + Eq,
	Timestamp: AtLeast32Bit + Parameter + Default + Debug + Copy,
{
	identity: AccountId,
	level: IdentityLevel,
	proof: PhysicalProof<Timestamp, ProofData>,
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	// type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
	type Timestamp: Parameter + Default + AtLeast32Bit
		+ Scale<Self::BlockNumber, Output = Self::Timestamp> + Copy;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Request a peer review to gain a specific IdentityLev
		#[weight = 10_000]
		fn request_peer_review(origin, identity_level: IdentityLevel, at: T::Timestamp) {
			let caller = ensure_signed(origin)?;
			Self::do_request_peer_review(caller, identity_level, at)?;
			// What happens here is that it either returns the Err(e) or Ok(()), DispatchResult is implicit
		}

		/// As a reviewer, approve a reviewed PhysicalIdentity by supplying a proof
		#[weight = 10_000]
		pub fn approve_identity(origin, review_process: Ticket<T>, proof_data: ProofType) {
			let _ = ensure_signed(origin)?;
			Self::do_approve_identity(review_process, proof_data)?;
		}
		
		/// As a reviewer, reject a reviewed PhysicalIdentity
		#[weight = 10_000]
		pub fn reject_identity(origin, review_process: Ticket<T>) {
			let _ = ensure_signed(origin)?;
			Self::do_reject_identity(review_process)?;
		}

		/// As a participant, report a missing participant
		#[weight = 10_000]
		pub fn report_missing(origin, review_process: Ticket<T>, missing: Vec<IdentityId<T>>) {
			let _ = ensure_signed(origin)?;
			Self::do_report_missing(review_process, missing)?;
		}
	}
}

impl<T: Trait> Module<T> {
	fn do_request_peer_review(user: T::AccountId, _identity_level: IdentityLevel, _at: T::Timestamp)
		-> Result<T::AccountId, DispatchError>
	{
		// TODO implement
		Ok(user)
	}

	fn do_approve_identity(_review_process: Ticket<T>, _proof_data: ProofType)
		-> Result<(), DispatchError>
	{
		// TODO implement
		Ok(())
	}

	fn do_reject_identity(_review_process: Ticket<T>) -> Result<(), DispatchError> {
		// TODO implement
		Ok(())
	}

	fn do_report_missing(_review_process: Ticket<T>, _missing: Vec<IdentityId<T>>)
		-> Result<(), DispatchError>
	{
		Ok(())
	}

	fn do_get_appointments(_identity: &IdentityId<T>) -> Vec<(T::Timestamp, Vec<IdentityId<T>>)> {
		Default::default()
	}

	fn do_get_identity_level(_identity: &IdentityId<T>) -> IdentityLevel {
		5
	}

	fn do_get_identity_id(address: &T::AccountId) -> IdentityId<T> {
		address.clone()
	}

	fn do_get_address(identity: &IdentityId<T>) -> T::AccountId {
		identity.clone()
	}
}

impl<T: Trait> traits::PeerReviewedPhysicalIdentity<ProofType> for Module<T> {
	type Address = T::AccountId;
	type Ticket = T::AccountId;
	type Timestamp = T::Timestamp;
	type IdentityLevel = IdentityLevel;
	type IdentityId = IdentityId<T>;

	/// Request a peer review to gain a specific IdentityLevel
	fn request_peer_review(user: Self::Address, identity_level: Self::IdentityLevel, at: Self::Timestamp)
		-> Result<Self::Ticket, DispatchError>
	{
		Self::do_request_peer_review(user, identity_level, at)
	}

	/// As a reviewer, approve a reviewed PhysicalIdentity by supplying a proof
	fn approve_identity(review_process: Self::Ticket, proof_data: ProofType)
		-> Result<(), DispatchError>
	{
		Self::do_approve_identity(review_process, proof_data)
	}

	/// As a reviewer, reject a reviewed PhysicalIdentity
	fn reject_identity(review_process: Self::Ticket) -> Result<(), DispatchError> {
		Self::do_reject_identity(review_process)
	}

	/// As a participant, report a missing participant
	fn report_missing(review_process: Self::Ticket, missing: Vec<Self::IdentityId>)
		-> Result<(), DispatchError>
	{
		Self::do_report_missing(review_process, missing)
	}

	/// Get the appointments for a DDI (when the DDI has to participate in an audit)
	fn get_appointments(identity: &Self::IdentityId) -> Vec<(Self::Timestamp, Vec<Self::IdentityId>)> {
		Self::do_get_appointments(identity)
	}


	/// Receive the identity level of a specific PhysicalIdentity.
	fn get_identity_level(identity: &Self::IdentityId) -> Self::IdentityLevel {
		// TODO: implement
		Self::do_get_identity_level(identity)
	}

	/// Get IdentityId for an address
	fn get_identity_id(address: &Self::Address) -> Self::IdentityId {
		Self::do_get_identity_id(address)
	}

	/// Get (main) address for an IdentityId
	fn get_address(identity: &Self::IdentityId) -> Self::Address {
		Self::do_get_address(identity)
	}
}
