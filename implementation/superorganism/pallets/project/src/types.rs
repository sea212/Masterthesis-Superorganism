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

use frame_support::dispatch::{Codec, Decode, Encode, EncodeLike, fmt::Debug, Vec};
use pallet_proposal_types::ProposalWinner;
#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};


// Important: Change Vec<u8> to a fixed length type (otherwise attackable)
pub type DocumentCID = Vec<u8>;
pub type ProposalCID = Vec<u8>;
pub type ConcernCID = ProposalCID;
pub type ProjectID = u64;

/// Contains all relevant information regarding a worker
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Worker<Balance, BlockNumber, IdentityId> where
	Balance: Codec + Clone + Debug + Eq + PartialEq,
	BlockNumber: Codec + Clone + Debug + Eq + PartialEq,
	IdentityId: Codec + Clone + Debug + Eq + EncodeLike,
{
	worker: IdentityId,
	job_description: DocumentCID,
	salary: Balance,
	hired: BlockNumber,
}

impl<Balance, BlockNumber, IdentityId> Worker<Balance, BlockNumber, IdentityId> where
	Balance: Codec + Clone + Debug + Eq + PartialEq,
	BlockNumber: Codec + Clone + Debug + Eq + PartialEq,
	IdentityId: Codec + Clone + Debug + Eq + EncodeLike,
{
	pub fn new(worker: IdentityId, job_description: DocumentCID,
				salary: Balance, hired: BlockNumber) -> Self 
	{
		Worker{worker, job_description, salary, hired}
	}
}

impl<Balance, BlockNumber, IdentityId> Default for Worker<Balance, BlockNumber, IdentityId> where
	Balance: Codec + Clone + Debug + Default + Eq + PartialEq,
	BlockNumber: Codec + Clone + Debug + Default + Eq + PartialEq,
	IdentityId: Codec + Clone + Debug + Default + Eq + EncodeLike,
{
	fn default() -> Self {
		Worker::new(Default::default(), Default::default(), Default::default(), Default::default())
	}
}


/// Contains all relevant information for a project
#[derive(Clone, Decode, Debug, Encode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Project<Balance, BlockNumber, IdentityId> where
	Balance: Codec + Clone + Debug + Eq + PartialEq,
	BlockNumber: Codec + Clone + Debug + Eq + PartialEq,
	IdentityId: Codec + Clone + Debug + Eq + EncodeLike,
{
	pub id: ProjectID,
	pub proposal: ProposalWinner<IdentityId>,
	pub project_leader: Option<Worker<Balance, BlockNumber, IdentityId>>,
	pub open_positions: Vec<DocumentCID>,
	pub workers: Vec<Worker<Balance, BlockNumber, IdentityId>>, // Maybe HashMap?
	pub deadline: BlockNumber,
}

impl<Balance, BlockNumber, IdentityId> Project<Balance, BlockNumber, IdentityId> where
	Balance: Codec + Clone + Debug + Default + Eq + PartialEq,
	BlockNumber: Codec + Clone + Debug + Default + Eq + PartialEq,
	IdentityId: Codec + Clone + Debug + Default + Eq + EncodeLike,
{
	pub fn new(id: ProjectID, proposal: ProposalWinner<IdentityId>) -> Self {
		Project{id, proposal, project_leader: None, workers: Default::default(),
				open_positions: Default::default(), deadline: Default::default()}
	}
}

impl<Balance, BlockNumber, IdentityId> Default for Project<Balance, BlockNumber, IdentityId> where
	Balance: Codec + Clone + Debug + Default + Eq + PartialEq,
	BlockNumber: Codec + Clone + Debug + Default + Eq + PartialEq,
	IdentityId: Codec + Clone + Debug + Default + Eq + EncodeLike,
{
	fn default() -> Self {
		Project::new(Default::default(), Default::default())
	}
}
