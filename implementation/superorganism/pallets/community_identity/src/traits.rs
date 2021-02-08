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

use frame_support::{
	dispatch::{Codec, Decode, DispatchError, Encode, EncodeLike, fmt::Debug, Parameter, Vec},
	sp_runtime::traits::AtLeast32Bit,
};
use num_traits::Num;

/// Trait for identity modules that want to support peer reviewed physical identities
///
/// Example process:
/// 1. User requests review for level <n>
/// 2. n random authorized reviewers (e.g. ```IdentityLevel``` > l) are selected
/// 3. Offchain: Select date and video chat plattform
/// 4. Review process: Reviewers check if the person knows what is happening and if its a realtime transmission
/// 5. Reviewers use software to determine biometric data
/// 6. Reviewers call ```approve_identity``` (including biometric data) or ```reject_identity```
///
/// Important: Force every user to redo this process in a specified period or reduce identity level to 1
/// due to possibly changing biometric data
///
/// To solve: How to handle twins.
///
/// Note: request_peer_review must lock and potentially burn coins to avoid DDoS
pub trait PeerReviewedPhysicalIdentity<ProofData>
	where ProofData: Codec + Clone + Debug + Decode + Encode + Eq + PartialEq
{
	type Address: Codec + Clone + Eq + EncodeLike + Debug;
	type Ticket: Codec + Clone + Eq + EncodeLike + Debug;
	type Timestamp: AtLeast32Bit + Parameter + Default + Debug + Copy;
	type IdentityLevel: Num;
	type IdentityId: Codec + Clone + Eq + EncodeLike + Debug;

	/// Request a peer review to gain a specific IdentityLevel
	fn request_peer_review(user: Self::Address, identity_level: Self::IdentityLevel, at: Self::Timestamp) 
		-> Result<Self::Ticket, DispatchError>;
	/// As a reviewer, approve a reviewed PhysicalIdentity by supplying a proof
	fn approve_identity(review_process: Self::Ticket, proof_data: ProofData) -> Result<(), DispatchError>;
	/// As a reviewer, reject a reviewed PhysicalIdentity
	fn reject_identity(review_process: Self::Ticket) -> Result<(), DispatchError>;
	/// As a participant, report a missing participant
	fn report_missing(review_process: Self::Ticket, missing: Vec<Self::IdentityId>) -> Result<(), DispatchError>;
	/// Get the appointments for a DDI (when the DDI has to participate in an audit)
	fn get_appointments(identity: &Self::IdentityId) -> Vec<(Self::Timestamp, Vec<Self::IdentityId>)>;
	/// Receive the identity level of a specific PhysicalIdentity.
	fn get_identity_level(identity: &Self::IdentityId) -> Self::IdentityLevel;
	/// Get IdentityId for an address
	fn get_identity_id(address: &Self::Address) -> Self::IdentityId;
	/// Get (main) address for an IdentityId
	fn get_address(identity: &Self::IdentityId) -> Self::Address;
}
