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

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use sp_runtime::{RuntimeDebug, traits::Zero};
use sp_std::{iter::from_fn, prelude::*, marker::PhantomData};

use crate::traits::{
	OnHeadersSubmitted, HeaderChain, Header, Storage,
	HeaderFor, HeaderAuxFor, BlockNumberFor, BlockHashFor,
};

#[derive(Encode, Decode)]
pub struct Configuration {
	pub signed_transactions: bool,
	pub unsigned_transactions: bool,
}

/// The module configuration trait
pub trait Trait: frame_system::Trait {
	type BridgeTypes: HeaderChain;
	/// Handler for headers submission result.
	type OnHeadersSubmitted: OnHeadersSubmitted<Self::AccountId>;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		pub fn import_header(
			origin,
			header: HeaderFor<T::BridgeTypes>,
			aux: HeaderAuxFor<T::BridgeTypes>,
		) {
			// TODO [ToDr[ Check configuration
			let submitter = frame_system::ensure_signed(origin)?;
			let import_result: Result<_, String> = unimplemented!();
			// let import_result = import::import_headers(
			// 	&mut BridgeStorage,
			// 	&kovan_aura_config(),
			// 	&kovan_validators_config(),
			// 	crate::import::PRUNE_DEPTH,
			// 	headers_with_receipts,
			// );

			match import_result {
				Ok((useful, useless)) =>
					T::OnHeadersSubmitted::on_valid_headers_submitted(submitter, useful, useless),
				Err(error) => {
					// even though we may have accept some headers, we do not want to reward someone
					// who provides invalid headers
					T::OnHeadersSubmitted::on_invalid_headers_submitted(submitter);
					return Err(error.into());
				},
			}
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Bridge {
		/// Best known block.
		BestBlock: (BlockNumberFor<T::BridgeTypes>, BlockHashFor<T::BridgeTypes>);
		/// Best finalized block.
		FinalizedBlock: (BlockNumberFor<T::BridgeTypes>, BlockHashFor<T::BridgeTypes>);
		/// Oldest unpruned block(s) number.
		OldestUnprunedBlock: BlockNumberFor<T::BridgeTypes>;
		/// Map of imported headers by hash.
		Headers: map hasher(blake2_256) BlockHashFor<T::BridgeTypes> =>
			Option<HeaderFor<T::BridgeTypes>>;
		/// Map of imported header hashes by number.
		HeadersByNumber: map hasher(blake2_256) BlockNumberFor<T::BridgeTypes> => Option<Vec<BlockHashFor<T::BridgeTypes>>>;
/* TODO [Validator Sets?]
		/// The ID of next validator set.
		NextValidatorsSetId: u64;
		/// Map of validators sets by their id.
		ValidatorsSets: map hasher(blake2_256) u64 => Option<(H256, Vec<Address>)>;
		/// Validators sets reference count. Each header that is authored by this set increases
		/// the reference count. When we prune this header, we decrease the reference count.
		/// When it reaches zero, we are free to prune validator set as well.
		ValidatorsSetsRc: map hasher(blake2_256) u64 => Option<u64>;
		/// Map of validators set changes scheduled by given header.
		ScheduledChanges: map hasher(blake2_256) H256 => Option<Vec<Address>>;
*/
	}
	add_extra_genesis {
		config(initial_header): HeaderFor<T::BridgeTypes>;
		// config(initial_difficulty): U256;
		// config(initial_validators): Vec<Address>;
		build(|config| {
			// the initial blocks should be selected so that:
			// 1) it doesn't signal validators changes;
			// 2) there are no scheduled validators changes from previous blocks;
			// 3) (implied) all direct children of initial block are authred by the same validators set.

			// assert!(
			// 	!config.initial_validators.is_empty(),
			// 	"Initial validators set can't be empty",
			// );

			let initial_hash = config.initial_header.compute_hash();
			// BestBlock::put((config.initial_header.number, initial_hash, config.initial_difficulty));
			BestBlock::<T>::put((config.initial_header.number(), initial_hash));
			FinalizedBlock::<T>::put((config.initial_header.number(), initial_hash));
			OldestUnprunedBlock::<T>::put(config.initial_header.number());
			HeadersByNumber::<T>::insert(config.initial_header.number(), vec![initial_hash]);
			Headers::<T>::insert(initial_hash, config.initial_header.clone());
			// StoredHeader {
			// 	header: config.initial_header.clone(),
			// 	total_difficulty: config.initial_difficulty,
			// 	next_validators_set_id: 0,
			// });
			// NextValidatorsSetId::put(1);
			// ValidatorsSets::insert(0, (initial_hash, config.initial_validators.clone()));
			// ValidatorsSetsRc::insert(0, 1);
		})
	}
}

impl<T: Trait> Module<T> {
	/// Returns number and hash of the best block known to the bridge module.
	/// The caller should only submit `import_header` transaction that makes
	/// (or leads to making) other header the best one.
	pub fn best_block() -> (BlockNumberFor<T::BridgeTypes>, BlockHashFor<T::BridgeTypes>) {
		BridgeStorage::<T>::default().best_block()
	}
	// TODO [ToDr] remove?
    //
	// /// Returns true if the import of given block requires transactions receipts.
	// pub fn is_import_requires_receipts(header: Header) -> bool {
	// 	import::header_import_requires_receipts(&BridgeStorage, &kovan_validators_config(), &header)
	// }
    //
	// /// Returns true if header is known to the runtime.
	// pub fn is_known_block(hash: H256) -> bool {
	// 	BridgeStorage.header(&hash).is_some()
	// }
}

/// Runtime bridge storage.
struct BridgeStorage<T>(PhantomData<T>);

impl<T: Trait> Default for BridgeStorage<T> {
	fn default() -> Self {
		BridgeStorage(Default::default())
	}
}

impl<T: Trait> Storage<T::BridgeTypes> for BridgeStorage<T> {
	fn best_block(&self) -> (BlockNumberFor<T::BridgeTypes>, BlockHashFor<T::BridgeTypes>) {
		BestBlock::<T>::get()
	}

	fn finalized_block(&self) -> (BlockNumberFor<T::BridgeTypes>, BlockHashFor<T::BridgeTypes>) {
		FinalizedBlock::<T>::get()
	}

	fn header(&self, hash: &BlockHashFor<T::BridgeTypes>) -> Option<HeaderFor<T::BridgeTypes>> {
		Headers::<T>::get(hash)
	}
    //
	// fn import_context(&self, parent_hash: &H256) -> Option<ImportContext> {
	// 	Headers::get(parent_hash).map(|parent_header| {
	// 		let (next_validators_set_start, next_validators) =
	// 			ValidatorsSets::get(parent_header.next_validators_set_id)
	// 				.expect("validators set is only pruned when last ref is pruned; there is a ref; qed");
	// 		ImportContext {
	// 			parent_header: parent_header.header,
	// 			parent_total_difficulty: parent_header.total_difficulty,
	// 			next_validators_set_id: parent_header.next_validators_set_id,
	// 			next_validators_set: (next_validators_set_start, next_validators),
	// 		}
	// 	})
	// }
    //
	// fn scheduled_change(&self, hash: &H256) -> Option<Vec<Address>> {
	// 	ScheduledChanges::get(hash)
	// }

	fn insert_header(
		&mut self,
		is_best: bool,
		number: BlockNumberFor<T::BridgeTypes>,
		hash: BlockHashFor<T::BridgeTypes>,
		header: HeaderFor<T::BridgeTypes>,
	) {
		if is_best {
			BestBlock::<T>::put((number, hash));
		}
		// if let Some(scheduled_change) = header.scheduled_change {
		// 	ScheduledChanges::insert(&header.hash, scheduled_change);
		// }
		// let next_validators_set_id = match header.enacted_change {
		// 	Some(enacted_change) => {
		// 		let next_validators_set_id = NextValidatorsSetId::mutate(|set_id| {
		// 			let next_set_id = *set_id;
		// 			*set_id += 1;
		// 			next_set_id
		// 		});
		// 		ValidatorsSets::insert(next_validators_set_id, (header.hash, enacted_change));
		// 		ValidatorsSetsRc::insert(next_validators_set_id, 1);
		// 		next_validators_set_id
		// 	}
		// 	None => {
		// 		ValidatorsSetsRc::mutate(header.context.next_validators_set_id, |rc| {
		// 			*rc = Some(rc.map(|rc| rc + 1).unwrap_or(1));
		// 			*rc
		// 		});
		// 		header.context.next_validators_set_id
		// 	}
		// };

		HeadersByNumber::<T>::append_or_insert(number, vec![hash]);
		Headers::<T>::insert(
			&hash,
			header,
			// StoredHeader {
			// 	header: header.header,
			// 	total_difficulty: header.total_difficulty,
			// 	next_validators_set_id,
			// },
		);
	}

	fn finalize_headers(
		&mut self,
		finalized: Option<(BlockNumberFor<T::BridgeTypes>, BlockHashFor<T::BridgeTypes>)>,
		prune_end: Option<BlockNumberFor<T::BridgeTypes>>,
	) {
		// remember just finalized block
		let finalized_number = finalized
			.as_ref()
			.map(|f| f.0)
			.unwrap_or_else(|| FinalizedBlock::<T>::get().0);
		if let Some(finalized) = finalized {
			FinalizedBlock::<T>::put(finalized);
		}

		// if let Some(prune_end) = prune_end {
		// 	let prune_begin = OldestUnprunedBlock::get();
        //
		// 	for number in prune_begin..prune_end {
		// 		let blocks_at_number = HeadersByNumber::take(number);
        //
		// 		// ensure that unfinalized headers we want to prune do not have scheduled changes
		// 		if number > finalized_number {
		// 			if let Some(ref blocks_at_number) = blocks_at_number {
		// 				if blocks_at_number.iter().any(|block| ScheduledChanges::contains_key(block)) {
		// 					HeadersByNumber::insert(number, blocks_at_number);
		// 					OldestUnprunedBlock::put(number);
		// 					return;
		// 				}
		// 			}
		// 		}
        //
		// 		// physically remove headers and (probably) obsolete validators sets
		// 		for hash in blocks_at_number.into_iter().flat_map(|x| x) {
		// 			let header = Headers::take(&hash);
		// 			ScheduledChanges::remove(hash);
		// 			if let Some(header) = header {
		// 				ValidatorsSetsRc::mutate(header.next_validators_set_id, |rc| match *rc {
		// 					Some(rc) if rc > 1 => Some(rc - 1),
		// 					_ => None,
		// 				});
		// 			}
		// 		}
		// 	}
        //
		// 	OldestUnprunedBlock::put(prune_end);
		// }
	}
}

/// Return iterator of given header ancestors.
pub(crate) fn ancestry<'a, T: Trait, S: Storage<T::BridgeTypes>>(
	storage: &'a S,
	header: &HeaderFor<T::BridgeTypes>,
) -> impl Iterator<Item = (BlockHashFor<T::BridgeTypes>, HeaderFor<T::BridgeTypes>)> + 'a {
	let mut parent_hash = header.parent_hash().clone();
	from_fn(move || {
		let header = storage.header(&parent_hash)?;
		if header.number().is_zero() {
			return None;
		}

		let hash = parent_hash.clone();
		parent_hash = header.parent_hash().clone();
		Some((hash, header))
	})
}
