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

//! # pallet-proposal
//! Manages proposal and concern rounds as well as the correspondant voting rounds


use frame_support::{decl_error, decl_module, decl_storage, decl_event, Parameter, ensure, /*print, debug,*/
	dispatch::{Vec, DispatchResult, Dispatchable, DispatchError},
	traits::{Get, Currency, ReservableCurrency,
		schedule::{Anon, DispatchTime, LOWEST_PRIORITY},
	},
	sp_std::collections::vec_deque::VecDeque,
	//weights::Weight,
};
use frame_system::{ensure_root, ensure_signed, RawOrigin::Root};
// use frame_system;
use codec::Codec;
// Fixed point arithmetic
use sp_arithmetic::Permill;
// Identity pallet
use pallet_community_identity::{ProofType, IdentityId, IdentityLevel, traits::PeerReviewedPhysicalIdentity};
use pallet_council::{BlockNumber, DocumentCID, Ticket, traits::Council};
use pallet_project::{types::{Project as ProjectType}, traits::ProjectTrait};
// Custom types
use pallet_proposal_types::{Concern, ConcernCID, Proposal, ProposalCID, ProposalWinner, States};
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	// Type trait constraints
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Type that manages balances
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Define the Scheduler type. Just implement (unamed) scheduling trait Anon
	type Scheduler: Anon<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;
	type Proposal: Parameter + Dispatchable<Origin=Self::Origin> + From<Call<Self>>;
	type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>> + Codec + Clone + Eq;

	/// Define Identity type. Must implement PeerReviewedPhysicalIdentity trait
	type Identity: PeerReviewedPhysicalIdentity<ProofType, IdentityId = IdentityId<Self>,
						IdentityLevel = IdentityLevel, Address = Self::AccountId>;

	/// Define Council type. Must implement Council trait
	type Council: Council<IdentityId = IdentityId<Self>, DocumentCID=DocumentCID,
							BlockNumber=BlockNumber<Self>, Ticket=Ticket>;

	// Define Project type. Must implement ProjectTrait trait
	type Project: ProjectTrait<Balance = BalanceOf<Self>, IdentityId = IdentityId<Self>,
					ProposalWinner=ProposalWinner<IdentityId<Self>>,
					Project=ProjectType<BalanceOf<Self>, Self::BlockNumber, IdentityId<Self>>>;

	// Parameters
	/// How long is an identified user locked out from submitting proposals / concerns
	/// for bad behaviour. Value in seconds.
	type IdentifiedUserPenality: Get<u32>;

	/// Part 1.1: Proposal state configuration
	// How many (slashable) funds must a simple User (no identity) lock to be able to propose?
	// type UserProposeFee: Get<BalanceOf<Self>>;

	/// How many proposals can be submitted per proposal round? (required for weight calculation)
	type ProposeCap: Get<u32>;
	
	/// How many proposals can an identified user submit per proposal round?
	type ProposeIdentifiedUserCap: Get<u8>;

	/// Which identity level is required to create a proposal?
	type ProposeIdentityLevel: Get<u8>;

	/// How high is the reward (%) for the proposer if the proposal is converted into a project?
	type ProposeReward: Get<Permill>;

	/// How long can proposals be submitted? Value in seconds.
	type ProposeRoundDuration: Get<Self::BlockNumber>;

	/// Part 1.2: Proposal voting state configuration
	/// How many votes (%) does a proposal require to be accepted for the next round?
	type ProposeVoteAcceptanceMin: Get<Permill>;

	/// How long can votes for proposals be submitted?
	type ProposeVoteDuration: Get<Self::BlockNumber>;

	/// Which identity level (number of random verifications) is required to vote?
	type ProposeVoteIdentityLevel: Get<u8>;

	/// How many votes can each identified user (with an appropriate identity level) submit?
	type ProposeVoteMaxPerIdentifiedUser: Get<u16>;

	/// How high is the reward if a proposal that the user voted for passes into next round?
	type ProposeVoteCorrectReward: Get<BalanceOf<Self>>;

	/// Part 2.1: Concern state configuration
	/// How many concerns can be submitted per concern round? (required for weight calculation)
	type ConcernCap: Get<u32>;

	/// How many concerns can an identified user submit per concern round?
	type ConcernIdentifiedUserCap: Get<u8>;

	/// Which identity level is required to submit a concern?
	type ConcernIdentityLevel: Get<u8>;

	/// How high is the reward if the concern receives enough votes to be passed to the next state?
	type ConcernReward: Get<BalanceOf<Self>>;

	/// How long can concerns be submitted? Value in seconds.
	type ConcernRoundDuration: Get<Self::BlockNumber>;

	// How many (slashable) funds must a simple User (no identity) lock to be able to submit a concern?
	// type UserConcernFee: Get<BalanceOf<Self>>;

	/// Part 2.2: Concern voting state configuration
	/// How many votes (%) does a concern require to be accepted for the next round?
	type ConcernVoteAcceptanceMin: Get<Permill>;

	/// How long can votes for concerns be submitted?
	type ConcernVoteDuration: Get<Self::BlockNumber>;

	/// Which identity level (number of random verifications) is required to vote?
	type ConcernVoteIdentityLevel: Get<u8>;

	/// How many votes can each identified user (with an appropriate identity level) submit?
	type ConcernVoteMaxPerIdentifiedUser: Get<u16>;

	/// How high is the reward if a concern that the user voted for passes into next round?
	type ConcernVoteCorrectReward: Get<BalanceOf<Self>>;

	/// Part 3: Final evaluation of the winning proposals and associated concern by the council
	/// How much time is reserved for the council to vote? Value in seconds
	type CouncilVoteRoundDuration: Get<Self::BlockNumber>;

	/// How many percent of the council must agree that a concern is too serious to launch a
	/// project from the associated proposal?
	type CouncilAcceptConcernMinVotes: Get<Permill>;
}

// TODO: Remove pub storage and write getters
decl_storage! {
	trait Store for Module<T: Trait> as Proposal {
		/// The current proposal state
		// Note: We must specify config() for at least one storage item, otherwise
		// the state machine cannot be initialized during genesis, because
		// add_extra_genesis won't be called at all (1. Nov 2020)
		pub State get(fn state) config(): States = States::Uninitialized;
		/// BlockNumber for which the next state transit is scheduled
		pub NextTransit get(fn next_transit): T::BlockNumber = T::BlockNumber::from(0);
		/// Current round
		// decided for u8 because after 256 proposal rounds the old proposals should be converted
		// into projects already. In addition, the blockchain state can be inspected at any block.
		// Last, There is no gurantee that the proposals still exist in decentralized storage.
		pub Round get(fn round): u8 = 0;

		/// Identity -> Proposals
		pub Proposals get(fn proposals): map hasher(identity)
			IdentityId<T> => Vec<Proposal> = Vec::new();
		/// Proposal -> Identity
		pub ProposalToIdentity get(fn proposal_to_identity): map hasher(identity)
			ProposalCID => IdentityId<T> = IdentityId::<T>::default();
		/// Identity -> Votes (we have to keep track of the CIDs to reward the user)
		pub ProposalVotes get(fn votes): map hasher(identity)
			IdentityId<T> => Vec<ProposalCID> = Vec::new();
		/// Total votes
		pub ProposalVoteCount get(fn vote_count): u32 = 0;
		/// Total proposals
		pub ProposalCount get(fn proposal_count): u32 = 0;
		/// Proposal winner for specific round
		pub ProposalWinners get(fn proposal_winners): map hasher(identity)
			u8 => VecDeque<ProposalWinner<IdentityId<T>>> = VecDeque::new();

		/// Identity -> Concerns
		pub Concerns get(fn concerns): map hasher(identity)
			IdentityId<T> => Vec<Concern> = Vec::new();
		/// ConcernCID -> Identity
		pub ConcernToIdentity get(fn concern_to_identity): map hasher(identity)
			(ConcernCID, ProposalCID) => IdentityId<T> = IdentityId::<T>::default();
		/// Total Concerns
		pub ConcernCount get(fn concern_count): u32 = 0;

		/// Identity -> Votes for concerns (we have to keep track of the CIDs to reward the user)
		pub ConcernVotes get(fn votes_concern): map hasher(identity)
			IdentityId<T> => Vec<ConcernCID> = Vec::new();
		/// Total votes for concerns
		pub ConcernVoteCount get(fn vote_count_concern): u32 = 0;

		/// Tickets used as reference for council polls targeting proposals
		pub CouncilVoteTickets get(fn council_vote_tickets): Vec<Ticket> = Vec::new();
	}
	add_extra_genesis {
		build(|_| {
			let _ = <Module<T>>::do_state_transit();
		}); 
	}
}

decl_event! {
	pub enum Event<T> where Balance = BalanceOf<T>,
							ID = IdentityId<T>,
							PW = ProposalWinner<IdentityId<T>> {
		/// Rotated to the next state. \[NewState\]
		StateRotated(States),
		/// Total reward for correct votes after VoteProposal round \[Balance\]
		TotalProposalReward(Balance),
		/// Total reward for winning concerns and votes after VoteConcern round \[Balance\]
		TotalConcernReward(Balance),
		/// If the council decides to deny a proposal, announce the proposal
		/// and the votes \[ProposalWinner, Vec(id, vote)\]
		CouncilDeniedProposal(PW, Vec<(ID, bool)>),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Concern was already submitted by another person
		ConcernAlreadySubmitted,
		/// Unable to add proposal because the concern limit is reached.
		ConcernLimitReached,
		/// Concern does not exist
		ConcernNotExistant,
		/// Identity level too low.
		IdentityLevelTooLow,
		/// Proposal was already submitted by another person
		ProposalAlreadySubmitted,
		/// Proposal does not exist
		ProposalNotExistant,
		/// Unable to add proposal because the proposal limit is reached.
		ProposalLimitReached,
		/// User submitted too many concerns.
		UserConcernLimitReached,
		/// User voted too many times on concerns.
		UserConcernVoteLimitReached,
		/// User submitted too many proposals.
		UserProposalLimitReached,
		/// User voted too many times.
		UserProposalVoteLimitReached,
		/// The operation requested cannot be executed because the pallet is in the wrong state.
		WrongState,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		// TODO: Think about how to handle arbitrarily huge number of votes
		// Fetch configuration
		/// How long is an identified user locked out from submitting proposals / concerns
		/// for bad behaviour. Value in seconds.
		const IdentifiedUserPenality: u32 = T::IdentifiedUserPenality::get() as u32;

		// Part 1.1: Proposal state configuration
		// How many (slashable) funds must a simple User (no identity) lock to be able to propose?
		// const UserProposeFee: BalanceOf<T> = T::UserProposeFee::get();

		/// How many proposals can be submitted per proposal round? (required for weight calculation)
		const ProposeCap: u32 = T::ProposeCap::get() as u32;
		
		/// How many proposals can an identified user submit per proposal round?
		const ProposeIdentifiedUserCap: u8 = T::ProposeIdentifiedUserCap::get() as u8;

		/// Which identity level is required to create a proposal?
		const ProposeIdentityLevel: u8 = T::ProposeIdentifiedUserCap::get() as u8;

		/// How high is the reward (%) for the proposer if the proposal is converted into a project?
		const ProposeReward: Permill = T::ProposeReward::get();

		/// How long can proposals be submitted? Value in seconds.
		const ProposeRoundDuration: T::BlockNumber = T::ProposeRoundDuration::get();

		// Part 1.2: Proposal voting state configuration
		/// How many votes (%) does a proposal require to be accepted for the next round?
		const ProposeVoteAcceptanceMin: Permill = T::ProposeVoteAcceptanceMin::get() as Permill;

		/// How long can votes for proposals be submitted?
		const ProposeVoteDuration: T::BlockNumber = T::ProposeVoteDuration::get();

		/// Which identity level (number of random verifications) is required to vote?
		const ProposeVoteIdentityLevel: u8 = T::ProposeVoteIdentityLevel::get() as u8;

		/// How many votes can each identified user (with an appropriate identity level) submit?
		const ProposeVoteMaxPerIdentifiedUser: u16 = T::ProposeVoteMaxPerIdentifiedUser::get() as u16;

		/// How high is the reward if a proposal that the user voted for passes into next round?
		const ProposeVoteCorrectReward: BalanceOf<T> = T::ProposeVoteCorrectReward::get();

		/// How many concerns can be submitted per concern round? (required for weight calculation)
		const ConcernCap: u32 = T::ConcernCap::get() as u32;

		// Part 2.1: Concern state configuration
		/// How many concerns can an identified user submit per concern round?
		const ConcernIdentifiedUserCap: u8 = T::ConcernIdentifiedUserCap::get() as u8;

		/// Which identity level is required to submit a concern?
		const ConcernIdentityLevel: u8 = T::ConcernIdentityLevel::get() as u8;

		/// How high is the reward if the concern receives enough votes to be passed to the next state?
		const ConcernReward: BalanceOf<T> = T::ConcernReward::get();

		/// How long can concerns be submitted? Value in seconds.
		const ConcernRoundDuration: T::BlockNumber = T::ConcernRoundDuration::get();

		// How many (slashable) funds must a simple User (no identity) lock to be able to submit a concern?
		// const UserConcernFee: BalanceOf<T> = T::UserConcernFee::get();

		// Part 2.2: Concern voting state configuration
		/// How many votes (%) does a concern require to be accepted for the next round?
		const ConcernVoteAcceptanceMin: Permill = T::ConcernVoteAcceptanceMin::get() as Permill;

		/// How long can votes for concerns be submitted?
		const ConcernVoteDuration: T::BlockNumber = T::ConcernVoteDuration::get();

		/// Which identity level (number of random verifications) is required to vote?
		const ConcernVoteIdentityLevel: u8 = T::ConcernVoteIdentityLevel::get() as u8;

		/// How many votes can each identified user (with an appropriate identity level) submit?
		const ConcernVoteMaxPerIdentifiedUser: u16 = T::ConcernVoteMaxPerIdentifiedUser::get() as u16;

		/// How high is the reward if a concern that the user voted for passes into next round?
		const ConcernVoteCorrectReward: BalanceOf<T> = T::ConcernVoteCorrectReward::get();

		/// Part 3: Final evaluation of the winning proposals and associated concern by the council
		/// How much time is reserved for the council to vote? Value in seconds
		const CouncilVoteRoundDuration: T::BlockNumber = T::CouncilVoteRoundDuration::get();

		/// How many percent of the council must agree that a concern is too serious to launch a
		/// project from the associated proposal?
		const CouncilAcceptConcernMinVotes: Permill = T::CouncilAcceptConcernMinVotes::get() as Permill;
		

		/// If this module was added during a runtime upgrade, start the state machine
		// If you want to implement this feature, consider:
		// 1. This function is called before the runtime state is initialized, therefore
		// 	  we don't have access to the current block number. This means that we we cannot
		//    figure out when the scheduler should transit into the next state in do_state_transit() (31. Oct 2020)
		/*
		fn on_runtime_upgrade() -> Weight {
			if let States::Uninitialized = <State>::get() {
				let _ = Self::do_state_transit();
			}

			0
		}*/

		
		/// Enforce state transit
		// Only for test purposes. Will be deleted in the future.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(5000,3)]
		fn state_transit(origin) -> DispatchResult {
			// check and change the current state
			ensure_root(origin)?;
			Self::do_state_transit()
		}


		/// As an identified user, submit a concern
		#[weight = 10_000 + T::DbWeight::get().reads_writes(6,3)]
		fn concern(origin, concern: ConcernCID, proposal: ProposalCID) {
			let caller = ensure_signed(origin)?;
			// Ensure that the pallet is in the appropriate state
			ensure!(<State>::get() == States::Concern, Error::<T>::WrongState);
			// Ensure that the maximum concern count was not reached yet
			ensure!(<ConcernCount>::get() < T::ConcernCap::get().into(), Error::<T>::ConcernLimitReached);
			// Ensure the identity level is high enough to submit a concern.
			let id: IdentityId<T> = T::Identity::get_identity_id(&caller);
			ensure!(T::Identity::get_identity_level(&id) >= T::ConcernIdentityLevel::get().into(),
					Error::<T>::IdentityLevelTooLow
			);
			// Ensure the user has not surpassed the concern limit per user
			ensure!(<Concerns<T>>::get(&id).len() < T::ConcernIdentifiedUserCap::get().into(),
					Error::<T>::UserConcernLimitReached
			);
			// Ensure that the concern was not already submitted
			ensure!(<ConcernToIdentity<T>>::get((&concern, &proposal)) == IdentityId::<T>::default(),
					Error::<T>::ConcernAlreadySubmitted
			);
			Self::add_concern(id, concern, proposal);
		}


		/// As an identified user, submit a proposal
		#[weight = 10_000 + T::DbWeight::get().reads_writes(6,3)]
		fn propose(origin, proposal: ProposalCID) {
			let caller = ensure_signed(origin)?;
			// Ensure that the pallet is in the appropriate state
			ensure!(<State>::get() == States::Propose, Error::<T>::WrongState);
			// Ensure that the maximum proposal count was not reached yet
			ensure!(<ProposalCount>::get() < T::ProposeCap::get().into(), Error::<T>::ProposalLimitReached);
			// Ensure the identity level is high enough to propose.
			let id: IdentityId<T> = T::Identity::get_identity_id(&caller);
			ensure!(T::Identity::get_identity_level(&id) >= T::ProposeIdentityLevel::get().into(),
					Error::<T>::IdentityLevelTooLow
			);
			// Ensure the user has not surpassed the proposal limit per user
			ensure!(<Proposals<T>>::get(&id).len() < T::ProposeIdentifiedUserCap::get().into(),
					Error::<T>::UserProposalLimitReached
			);
			// Ensure that the proposal was not already submitted
			ensure!(<ProposalToIdentity<T>>::get(&proposal) == IdentityId::<T>::default(),
					Error::<T>::ProposalAlreadySubmitted
			);
			Self::add_proposal(id, proposal);
		}

		/// As an identified user, vote for a concern
		#[weight = 10_000 + T::DbWeight::get().reads_writes(6,3)]
		fn vote_concern(origin, concern: ConcernCID, proposal: ProposalCID) {
			let caller = ensure_signed(origin)?;
			// Ensure that the pallet is in the appropriate state
			ensure!(<State>::get() == States::VoteConcern, Error::<T>::WrongState);
			// Ensure that the concern exists
			let proposer: IdentityId<T> = <ConcernToIdentity<T>>::get((&concern, &proposal));
			ensure!(proposer != IdentityId::<T>::default(),
				Error::<T>::ConcernNotExistant
			);
			// Ensure the identity level is high enough to vote.
			let id: IdentityId<T> = T::Identity::get_identity_id(&caller);
			ensure!(T::Identity::get_identity_level(&id) >= T::ConcernVoteIdentityLevel::get().into(),
					Error::<T>::IdentityLevelTooLow
			);
			// Ensure the user has not surpassed the vote limit per user
			ensure!(<ConcernVotes<T>>::get(&id).len() < T::ConcernVoteMaxPerIdentifiedUser::get().into(),
					Error::<T>::UserConcernVoteLimitReached
			);

			// Optional: Ensure that the user did not already vote for the concern (design decision)
			Self::add_vote_concern(id, concern, proposal, proposer);
		}

		/// As an identified user, vote for a proposal
		#[weight = 10_000 + T::DbWeight::get().reads_writes(6,3)]
		fn vote_proposal(origin, proposal: ProposalCID) {
			let caller = ensure_signed(origin)?;
			// Ensure that the pallet is in the appropriate state
			ensure!(<State>::get() == States::VotePropose, Error::<T>::WrongState);
			// Ensure that the proposal exists
			let proposer: IdentityId<T> = <ProposalToIdentity<T>>::get(&proposal);
			ensure!(proposer != IdentityId::<T>::default(),
				Error::<T>::ProposalNotExistant
			);
			// Ensure the identity level is high enough to vote.
			let id: IdentityId<T> = T::Identity::get_identity_id(&caller);
			ensure!(T::Identity::get_identity_level(&id) >= T::ProposeVoteIdentityLevel::get().into(),
					Error::<T>::IdentityLevelTooLow
			);
			// Ensure the user has not surpassed the vote limit per user
			ensure!(<ProposalVotes<T>>::get(&id).len() < T::ProposeVoteMaxPerIdentifiedUser::get().into(),
					Error::<T>::UserProposalVoteLimitReached
			);

			// Optional: Ensure that the user did not already vote for the proposal (design decision)
			Self::add_vote_proposal(id, proposal, proposer);
		}

		/*
		#[weight = 10_000]
		fn test_identity_level(origin) {
			let caller = ensure_signed(origin)?;
			let identity: IdentityId<T> = caller;
			let identity_level : IdentityLevel = 0;
			let level: IdentityLevel = T::Identity::get_identity_level(identity).unwrap_or(identity_level);
			debug::info!("IdentityLevel: {:?}", level);
		}*/
	}
}

impl<T: Trait> Module<T> {
	/// Add concern to storage and update relevant storage values
	fn add_concern(id: IdentityId<T>, concern: ConcernCID, proposal: ProposalCID) {
		// Create proper Concern and add it to the users list of concerns
		let document = Concern::new(concern.clone(), proposal.clone());
		<Concerns<T>>::mutate(&id, |user_concerns| {
			user_concerns.push(document);
		});
		// Add mapping from (ConcernCID, ProposalCid) to identity
		ConcernToIdentity::<T>::insert((&concern, &proposal), &id);
		// Increment total concern count
		<ConcernCount>::mutate(|cc| *cc += 1);
	}

	fn add_council_poll(mut winners: VecDeque<ProposalWinner<IdentityId<T>>>) {
		let mut tickets: Vec<Ticket> = Vec::new();
		let transit_time: T::BlockNumber = T::CouncilVoteRoundDuration::get();

		// Add every proposal and its concerns to a freshly created council poll
		for winner in winners.iter_mut() {
			let mut documents: Vec<DocumentCID> = Vec::new();
			documents.push(winner.proposal.clone());
			documents.append(&mut winner.concerns);

			// TODO: Better error handling
			if let Ok(ticket) = T::Council::add_poll(documents, transit_time) {
				tickets.push(ticket);
			}
		}

		CouncilVoteTickets::put(tickets);
	}

	/// Add proposal to storage and update relevant storage values
	fn add_proposal(id: IdentityId<T>, proposal: ProposalCID) {
		// Create proper Proposal and add it to the users list of proposals
		let document = Proposal::new(proposal.clone());
		<Proposals<T>>::mutate(&id, |user_proposals| {
			user_proposals.push(document);
		});
		// Add mapping from proposalCID to identity
		ProposalToIdentity::<T>::insert(&proposal, &id);
		// Increment total proposal count
		<ProposalCount>::mutate(|pc| *pc += 1);
	}

	/// Add vote to storage and update relevant storage values
	fn add_vote_proposal(id: IdentityId<T>, proposal: ProposalCID, proposer: IdentityId<T>) {
		// Add proposalCID to id votes
		<ProposalVotes<T>>::mutate(&id, |vote_cids| {
			vote_cids.push(proposal.clone())
		});
		// Increment vote count within Proposal structure
		<Proposals<T>>::mutate(&proposer, |proposals| {
			if let Some(p) = proposals.iter_mut().find(|el| el.proposal == proposal) {
				p.votes += 1;
			}
			// TODO: Better error handling. What if storage got corrupted somehow?
		});
		// Increment total vote count
		// TODO: Overflow handling
		<ProposalVoteCount>::mutate(|vc| *vc += 1);
	}

	/// Add vote to storage and update relevant storage values
	fn add_vote_concern(id: IdentityId<T>, concern: ConcernCID, proposal: ProposalCID, proposer: IdentityId<T>) {
		// Add concernCID to id votes
		<ConcernVotes<T>>::mutate(&id, |vote_cids| {
			vote_cids.push(concern.clone())
		});
		// Increment vote count within Concern structure
		<Concerns<T>>::mutate(&proposer, |concerns| {
			if let Some(p) = concerns.iter_mut().find(|el| {
				el.concern == concern && el.associated_proposal == proposal
			}) {
				p.votes += 1;
			}
			// TODO: Better error handling. What if storage got corrupted somehow?
		});
		// Increment total vote count
		// TODO: Overflow handling
		<ConcernVoteCount>::mutate(|vc| *vc += 1);
	}

	/// Execute the state transit and schedule the next state transit
	fn do_state_transit() -> DispatchResult {
		let mut transit_time: T::BlockNumber = T::BlockNumber::from(0);

		// TODO: Early state transit when the proposal limit was reached.
		// TODO: Early state transition when every member of the council has voted.
		// TODO: Make Scheduler named and cancel any scheduled state transits before adding new.
		// TODO: Change mutate to get, checks values, and change them at the end of this function
		//			(verify first write last)
		let newstate: States = <State>::mutate(|state| {
			match state {
				States::Uninitialized => {
					*state = States::Propose;
					transit_time = T::ProposeRoundDuration::get();
				},
				States::Propose => {
					// Only transit state if proposals exist
					transit_time = T::ProposeRoundDuration::get();
					for _ in <Proposals<T>>::iter() {
						transit_time = T::ProposeVoteDuration::get();
						*state = States::VotePropose;
						break;
					}
				},
				States::VotePropose => {
					Self::evaluate_proposal_votes();
					let round = <Round>::get();

					// Start next proposal round if no proposal did receive enough votes
					if <ProposalWinners<T>>::get(round).len() == 0 {
						*state = States::Propose;
						transit_time = T::ProposeRoundDuration::get();
						if round == u8::MAX { Round::put(0); }
						else { Round::put(round+1); }
						return *state;
					}

					*state = States::Concern;
					transit_time = T::ConcernRoundDuration::get();
				},
				States::Concern => {
					// Skip VoteConcern if no concerns exist
					if <ConcernCount>::get() == 0 {
						// Add every proposal and its concerns to a freshly created council poll
						let round: u8 = <Round>::get();
						let winners: VecDeque<ProposalWinner<IdentityId<T>>> = <ProposalWinners<T>>::get(&round);
						Self::add_council_poll(winners);
						*state = States::VoteCouncil;
						transit_time = T::CouncilVoteRoundDuration::get();
					} else {
						transit_time = T::ConcernVoteDuration::get();
						*state = States::VoteConcern;
					}
				},
				States::VoteConcern => {
					// Determine winning concerns and add to associated winning proposals
					let winners: VecDeque<ProposalWinner<IdentityId<T>>> = Self::evaluate_concern_votes();
					// Add every proposal and its concerns to a freshly created council poll
					Self::add_council_poll(winners);
					transit_time = T::CouncilVoteRoundDuration::get();
					*state = States::VoteCouncil;
				},
				States::VoteCouncil => {
					let round = <Round>::get();
					let winners = <ProposalWinners<T>>::get(&round);

					// Get voting result and evaluate vote percentage
					for (idx, ticket) in <CouncilVoteTickets>::get().iter().enumerate() {
						// TODO: Better error handling (error = ticket number not found in council)
						if let Some(result) = T::Council::get_result(ticket) {
							let mut percentage_no = Permill::zero();
							let mut votes_no: u32 = 0;

							for _ in result.iter().filter(|v| v.1 == false) { votes_no += 1; }

							if result.len() != 0 {
								percentage_no = Permill::from_rational_approximation(
									votes_no, result.len() as u32
								);
							}

							// Spawn project from passed proposals
							if percentage_no < T::CouncilAcceptConcernMinVotes::get() {
								let _ = T::Project::spawn_project(winners[idx].clone());
							} else {
								Event::<T>::CouncilDeniedProposal(winners[idx].clone(), result);
							}
						}
					}

					// increment round and rotate state
					if round == u8::MAX { Round::put(0); }
					else { Round::put(round+1); }
					*state = States::Propose;
					transit_time = T::ProposeRoundDuration::get();
				}
			}
		*state
		});

		let current_block: T::BlockNumber = frame_system::Module::<T>::block_number();
		let next_state_transit: T::BlockNumber = current_block + transit_time;

		if T::Scheduler::schedule(
			DispatchTime::At(next_state_transit),
			None,
			LOWEST_PRIORITY,
			Root.into(),
			Call::state_transit().into(),
		).is_err() {
			// Todo: Appropriate Error or handling.
			return Err(DispatchError::Other("Setting anonymous scheduler for \"state_transit\" failed"));
		};

		NextTransit::<T>::put(next_state_transit);
		Self::deposit_event(Event::<T>::StateRotated(newstate));
		Ok(())
	}

	/// On state transit from VoteConcern, evaluate all concerns and votes and pay winners and correct voters.
	fn evaluate_concern_votes() -> VecDeque<ProposalWinner<IdentityId<T>>> {
		let total_votes: u32 = <ConcernVoteCount>::get();
		let round: u8 = <Round>::get();
		let mut winners: VecDeque<ProposalWinner<IdentityId<T>>> = <ProposalWinners<T>>::get(&round);
		let mut total_reward_issued = BalanceOf::<T>::from(0);
		let reward_propose: BalanceOf<T> = T::ConcernReward::get();
		let reward_vote: BalanceOf<T> = T::ConcernVoteCorrectReward::get();

		// Drain all Concerns and add winners into winner variable and into storage ProposalWinners
		for (id, concerns) in <Concerns<T>>::drain() {
			for concern in concerns.iter() {
				// Here we inspect every single concern of a specific user. Add it if it won.
				let mut vote_ratio = Permill::zero();

				if total_votes > 0 {
					vote_ratio = Permill::from_rational_approximation(concern.votes, total_votes);
				}

				if vote_ratio >= T::ConcernVoteAcceptanceMin::get() {
					if let Some(winner) = winners.iter_mut().find(|el| el.proposal == concern.associated_proposal) {
						winner.concerns.push(concern.concern.clone());

						if T::Currency::deposit_into_existing(&T::Identity::get_address(&id), reward_propose).is_ok() {
							total_reward_issued += reward_propose;
						}
					}
				}
			}
		}

		// Drain all voters ProposalVotes and reward them if the proposal they voted for won
		for (id, votes) in <ConcernVotes<T>>::drain() {
			for _ in votes.iter().filter(|v| {
				// Only count votes for winning proposals
				for winner in winners.iter() {
					for concern in winner.concerns.iter() {
						if *concern == **v { return true; }
					}
				}
				false
			}) {
				// TODO: When tx by identity is implemented, change to deposit_creating
				// (since identity does not require to spend fees for tx,
				// the account might not have been created on chain)
				// TODO: Error handling
				if T::Currency::deposit_into_existing(&T::Identity::get_address(&id), reward_vote).is_ok() {
					total_reward_issued += reward_vote;
				}
			}
		}

		ProposalWinners::<T>::insert(round, winners.clone());
		// Clear ProposalToIdentity, ProposalVoteCount, ProposalCount
		// Avoid collecting the iterator to avoid creating a new Vector
		ConcernToIdentity::<T>::drain().nth(usize::MAX);
		ConcernVoteCount::put(0);
		ConcernCount::put(0);
		Self::deposit_event(Event::<T>::TotalConcernReward(total_reward_issued));
		return winners;
	}


	/// On state transit from VotePropose, evaluate all proposals and votes and pay correct voters.
	fn evaluate_proposal_votes() {
		let total_votes: u32 = <ProposalVoteCount>::get();
		let round: u8 = <Round>::get();
		let mut winners: Vec<ProposalWinner<IdentityId<T>>> = Vec::new();
		let mut total_reward_issued = BalanceOf::<T>::from(0);
		let reward: BalanceOf<T> = T::ProposeVoteCorrectReward::get();

		// Drain all Proposals and put winners into winner variable and into storage ProposalWinners
		for (id, proposals) in <Proposals<T>>::drain() {
			for proposal in proposals.iter() {
				// Here we inspect every single proposal of a specific user. Add it if it won.
				let mut vote_ratio = Permill::zero();

				if total_votes > 0 {
					vote_ratio = Permill::from_rational_approximation(proposal.votes, total_votes);
				}

				if vote_ratio >= T::ProposeVoteAcceptanceMin::get() {
					let document = ProposalWinner::<IdentityId<T>>::new(
						Vec::new(), id.clone(), proposal.proposal.clone(), vote_ratio
					);
					winners.push(document);
				}
			}
		}

		winners.sort_by(|a, b| a.vote_ratio.cmp(&b.vote_ratio));
		ProposalWinners::<T>::insert(round, VecDeque::from(winners.clone()));
		// Drain all voters ProposalVotes and reward them if the proposal they voted for won
		for (id, votes) in <ProposalVotes<T>>::drain() {
			for _ in votes.iter().filter(|v| {
				// Only count votes for winning proposals
				for winner in winners.iter() {
					if winner.proposal == **v {
						return true;
					}
				}
				false
			}) {
				// TODO: When tx by identity is implemented, change to deposit_creating
				// (since identity does not require to spend fees for tx,
				// the account might not have been created on chain)
				// TODO: Error handling
				if T::Currency::deposit_into_existing(&T::Identity::get_address(&id), reward).is_ok() {
					total_reward_issued += reward;
				}
			}
		}

		// Clear ProposalToIdentity, ProposalVoteCount, ProposalCount
		// Avoid collecting the iterator to avoid creating a new Vector
		ProposalToIdentity::<T>::drain().nth(usize::MAX);
		ProposalVoteCount::put(0);
		ProposalCount::put(0);
		Self::deposit_event(Event::<T>::TotalProposalReward(total_reward_issued));
	}

	/*fn incr_round() {
		<Round>::mutate(|r| {
			if *r == u8::MAX { *r = 0; }
			else { *r += 1; }
		});
	}*/
}
