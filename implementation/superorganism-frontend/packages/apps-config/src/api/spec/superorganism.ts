export default {
  "Address": "AccountId",
  "LookupSource": "AccountId",
  "States": {
    "_enum": [
      "Uninitialized",
      "Propose",
      "VotePropose",
      "Concern",
      "VoteConcern",
      "VoteCouncil"
    ]
  },
  "ProposalCID": "Vec<u8>",
  "ConcernCID": "ProposalCID",
  "DocumentCID": "ProposalCID",
  "Proposal": {
    "proposal": "ProposalCID",
    "votes": "u32"
  },
  "Concern": {
    "associated_proposal": "ProposalCID",
    "concern": "ConcernCID",
    "votes": "u32"
  },
  "VecDeque": "Vec<ProposalWinner>",
  "ProposalWinner": {
    "concerns": "Vec<ConcernCID>",
    "proposer": "IdentityId",
    "proposal": "ProposalCID",
    "vote_ratio": "Permill"
  },
  "IdentityLevel": "u8",
  "ProofType": "[u8; 32]",
  "IdentityId": "AccountId",
  "Ticket": "u64",
  "ProjectID": "u64",
  "Worker": {
    "worker": "IdentityId",
    "job_description": "DocumentCID",
    "salary": "Balance",
    "hired": "BlockNumber"
  },
  "Project": {
    "id": "ProjectID",
    "proposal": "ProposalWinner",
    "project_leader": "Option<Worker>",
    "open_positions": "Vec<DocumentCID>",
    "workers": "Vec<Worker>",
    "deadline": "BlockNumber"
  },
  "PRJ": "Project",
  "ID": "IdentityId",
  "PW": "ProposalWinner",
  "Timestamp": "u64"
}
