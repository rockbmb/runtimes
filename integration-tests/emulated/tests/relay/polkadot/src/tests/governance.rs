// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::*;
use emulated_integration_tests_common::{accounts::{ALICE, BOB, CHARLIE, DAVE, EVE}, xcm_emulator::log};

use frame_support::traits::{schedule::DispatchTime, StorePreimage};

use pallet_conviction_voting::{AccountVote, Conviction, Vote};
use pallet_referenda::ReferendumInfoFor;

use polkadot_runtime::{governance::pallet_custom_origins::Origin, ConvictionVoting, Preimage, Referenda};
use polkadot_runtime_common::impls::VersionedLocatableAsset;

use polkadot_runtime_constants::currency::UNITS as DOT;

use frame_system::Pallet as System;

#[test]
fn referendum_lifecycle() {
	let polkadot_alice = Polkadot::account_id_of(ALICE);
	let polkadot_bob = Polkadot::account_id_of(BOB);
	let polkadot_charlie = Polkadot::account_id_of(CHARLIE);
	let polkadot_dave = Polkadot::account_id_of(DAVE);
	let polkadot_eve = Polkadot::account_id_of(EVE);

	Polkadot::execute_with(|| {
		type Runtime = <Polkadot as Chain>::Runtime;
		type RuntimeCall = <Polkadot as Chain>::RuntimeCall;
		type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;
		type Referenda = <Polkadot as PolkadotPallet>::Referenda;

		let asset_kind: VersionedLocatableAsset = VersionedLocatableAsset::V4 {
			location: Location::new(0, Here),
			asset_id: Location::here().into(),
		};

		// Create proposal on which referendum will be based
		let treasury_call = RuntimeCall::Treasury(pallet_treasury::Call::<Runtime>::spend {
			asset_kind: Box::new(asset_kind),
			amount: 100 * DOT,
			beneficiary: Box::new(Location::new(0, Into::<[u8; 32]>::into(polkadot_bob.clone())).into()),
			valid_from: None,
		});

		let proposal = <Preimage as StorePreimage>::bound(treasury_call).unwrap();

		let alice_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_alice.clone());
		let bob_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_alice.clone());

		// Submit referendum
		assert_ok!(
			Referenda::submit(
				alice_origin.clone(),
				Box::new(Origin::SmallTipper.into()),
				proposal,
				DispatchTime::After(0u32.into())
			)
		);

		let ref_index: u32 = 0;
		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::Referenda(pallet_referenda::Event::Submitted { index: 0, .. }) => {},
			]
		);

		// Place decision deposit
		assert_ok!(
			Referenda::place_decision_deposit(
				bob_origin.clone(),
				ref_index
			)
		);

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::Referenda(pallet_referenda::Event::DecisionDepositPlaced { index: 0, .. }) => {},
			]
		);

		let k = <Polkadot as Chain>::System::block_number();
		log::warn!(target: "integration_tests", "Current block number: {:?}", k);

		// Vote on the referendum - use all three different voting kinds.

		let charlie_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_charlie.clone());
		let aye_vote = 5 * DOT;

		assert_ok!(
			ConvictionVoting::vote(
				charlie_origin.clone(),
				ref_index,
				AccountVote::Standard {
					vote: Vote {
						aye: true, conviction: Conviction::Locked1x
					},
					balance: aye_vote
				}
			)
		);

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::ConvictionVoting(pallet_conviction_voting::Event::Voted { vote: AccountVote::Standard { .. }, .. }) => {},
			]
		);

		// Split voting

		let dave_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_dave.clone());
		let nay_vote = 1 * DOT;

		assert_ok!(
			ConvictionVoting::vote(
				dave_origin.clone(),
				ref_index,
				AccountVote::Split {
					aye: aye_vote,
					nay: nay_vote
				}
			)
		);

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::ConvictionVoting(pallet_conviction_voting::Event::Voted { vote: AccountVote::Split { .. }, .. }) => {},
			]
		);

		// Split abstain voting

		let eve_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_eve.clone());
		let abstain_vote = 2 * DOT;

		assert_ok!(
			ConvictionVoting::vote(
				eve_origin.clone(),
				ref_index,
				AccountVote::SplitAbstain {
					aye: aye_vote,
					nay: nay_vote,
					abstain: abstain_vote
				}
			)
		);

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::ConvictionVoting(pallet_conviction_voting::Event::Voted { vote: AccountVote::SplitAbstain { .. }, .. }) => {},
			]
		);

		let k = <Polkadot as Chain>::System::block_number();
		log::warn!(target: "integration_tests", "Current block number: {:?}", k);

		let info = ReferendumInfoFor::<Runtime, ()>::get(ref_index).unwrap();
		log::warn!(target: "integration_tests", "Referendum info: {:?}", info);

		let root = <Polkadot as Chain>::RuntimeOrigin::root();
		//  Nudge the referendum to its next state
		assert_ok!(
			Referenda::nudge_referendum(root, ref_index)
		);

		// Advance to the end of the voting period
		<Polkadot as Chain>::System::set_block_number(k + 11);

		let k = <Polkadot as Chain>::System::block_number();
		log::warn!(target: "integration_tests", "Current block number: {:?}", k);

		System::set_block_number(current_block + 1);
		System::on_initialize(current_block + 1);

		let info = ReferendumInfoFor::<Runtime, ()>::get(ref_index).unwrap();
		log::warn!(target: "integration_tests", "Referendum info: {:?}", info);
	});

}
