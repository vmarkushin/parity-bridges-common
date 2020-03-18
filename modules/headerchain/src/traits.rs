// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

use sp_std::prelude::*;
use sp_runtime::traits::Zero;
use frame_support::dispatch::Parameter;

pub type HeaderFor<T> = <T as HeaderChain>::Header;
pub type HeaderAuxFor<T> = <T as HeaderChain>::HeaderAuxiliaryData;
pub type BlockNumberFor<T> = <<T as HeaderChain>::Header as Header>::Number;
pub type BlockHashFor<T> = <<T as HeaderChain>::Header as Header>::Hash;

pub trait Header: Parameter + Default /* TODO [ToDr] Required because of storage? */{
	type Number: Default + Parameter + Zero;
	type Hash: Default + Parameter;

	fn compute_hash(&self) -> Self::Hash;

	fn number(&self) -> &Self::Number;

	fn parent_hash(&self) -> &Self::Hash;
}

pub trait HeaderChain {
	type Header: Header;
	type HeaderAuxiliaryData: Parameter;
	type Event;
	type EventProofData;
	type Transaction;
	type TransactionProofData;

	// fn verify_events(
	// 	at: &Self::BlockHash,
	// 	events: Vec<Self::Event>,
	// 	proof_data: Self::EventProofData,
	// );
    //
	// fn verify_transactions(
	// 	at: &Self::BlockHash,
	// 	transactions: Vec<Self::Transactions>,
	// 	proof_data: Self::TransactionProofData,
	// );
    //
	// fn header(
	// 	at: &Self::BlockHash,
	// ) -> Option<Self::Header>;
    //
	// fn import_header(
	// 	header: Self::Header,
	// 	aux: Self::HeaderAuxiliaryData,
	// ) -> Result<Self::BlockHash, ()>;
}

/// The storage that is used by the client.
///
/// Storage modification must be discarded if block import has failed.
pub trait Storage<T: HeaderChain> {
	/// Get best known block.
	fn best_block(&self) -> (BlockNumberFor<T>, BlockHashFor<T>);
	/// Get last finalized block.
	fn finalized_block(&self) -> (BlockNumberFor<T>, BlockHashFor<T>);
	/// Get imported header by its hash.
	fn header(&self, hash: &BlockHashFor<T>) -> Option<T::Header>;
	// /// Get header import context by parent header hash.
	// fn import_context(&self, parent_hash: &H256) -> Option<ImportContext>;
	// /// Get new validators that are scheduled by given header.
	// fn scheduled_change(&self, hash: &H256) -> Option<Vec<Address>>;
	/// Insert imported header.
	fn insert_header(
		&mut self,
		is_best: bool,
		number: BlockNumberFor<T>,
		hash: BlockHashFor<T>,
		header: T::Header,
	);
	/// Finalize given block and prune all headers with number < prune_end.
	/// The headers in the pruning range could be either finalized, or not.
	/// It is the storage duty to ensure that unfinalized headers that have
	/// scheduled changes won't be pruned until they or their competitors
	/// are finalized.
	fn finalize_headers(
		&mut self,
		finalized: Option<(BlockNumberFor<T>, BlockHashFor<T>)>,
		prune_end: Option<BlockNumberFor<T>>,
	);
}

/// Decides whether the session should be ended.
pub trait OnHeadersSubmitted<AccountId> {
	/// Called when valid headers have been submitted.
	fn on_valid_headers_submitted(submitter: AccountId, useful: u64, useless: u64);

	/// Called when invalid headers have been submitted.
	fn on_invalid_headers_submitted(submitter: AccountId);
}

impl<AccountId> OnHeadersSubmitted<AccountId> for () {
	fn on_valid_headers_submitted(_submitter: AccountId, _useful: u64, _useless: u64) {}

	fn on_invalid_headers_submitted(_submitter: AccountId) {}
}

