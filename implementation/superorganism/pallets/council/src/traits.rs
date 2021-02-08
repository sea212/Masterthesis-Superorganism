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
use num_traits::Num;

/// Beginning of the council trait definition
/// Members of the council should be elected every <period>, should be able to be voted of
/// by the public, should resolve conflics and get rewarded if they do so or punished
/// if they don't fullfil their duty.
pub trait Council
{
	type IdentityId: Codec + Clone + Eq + EncodeLike + Debug;
	type Ticket: Num;
	type BlockNumber: Codec + Clone + Debug + Eq + PartialEq;
	type DocumentCID: Codec + Clone + Debug + Eq + PartialEq;

	/// As an identified user, vote for a council member
	fn vote_council_member(voter: Self::IdentityId, candidate: Self::IdentityId) -> Result<(), DispatchError>;
	/// As an identified user, vote to reelect council
	fn vote_reelect_council(voter: Self::IdentityId) -> Result<(), DispatchError>;
	/// As an identified user, vote to reelect a specific council member
	fn vote_reelect(voter: Self::IdentityId, member: Self::IdentityId) -> Result<(), DispatchError>;
	/// As root, queue a poll
	fn add_poll(documents: Vec<Self::DocumentCID>, until: Self::BlockNumber) -> Result<Self::Ticket, DispatchError>;
	/// As a council member, vote for a poll
	fn vote_poll(member: Self::IdentityId, poll: Self::Ticket, accept: bool) -> Result<(), DispatchError>;
	/// Retrieve result of a poll
	fn get_result(poll: &Self::Ticket) -> Option<Vec<(Self::IdentityId, bool)>>;
	// TODO
}
