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
use emulated_integration_tests_common::accounts::{ALICE, BOB, CHARLIE, DAVE};

use frame_support::{sp_runtime::traits::Dispatchable, traits::{schedule::DispatchTime, StorePreimage}};

use polkadot_runtime::{governance::pallet_custom_origins::Origin, Preimage, Referenda};
use polkadot_runtime_common::impls::VersionedLocatableAsset;

use polkadot_runtime_constants::currency::UNITS as DOT;

#[test]
fn referendum_lifecycle() {
	let polkadot_alice = Polkadot::account_id_of(ALICE);
	let polkadot_bob = Polkadot::account_id_of(BOB);
	let polkadot_charlie = Polkadot::account_id_of(CHARLIE);
	let polkadot_dave = Polkadot::account_id_of(DAVE);

	Polkadot::execute_with(|| {
		type Runtime = <Polkadot as Chain>::Runtime;
		type RuntimeCall = <Polkadot as Chain>::RuntimeCall;
		type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;
		type Referenda = <Polkadot as PolkadotPallet>::Referenda;

		let asset_kind: VersionedLocatableAsset = VersionedLocatableAsset::V4 {
			location: Location::new(0, Here),
			asset_id: Location::here().into(),
		};

		let treasury_call = RuntimeCall::Treasury(pallet_treasury::Call::<Runtime>::spend {
			asset_kind: Box::new(asset_kind),
			amount: 100 * DOT,
			beneficiary: Box::new(Location::new(0, Into::<[u8; 32]>::into(polkadot_bob.clone())).into()),
			valid_from: None,
		});

		let proposal = <Preimage as StorePreimage>::bound(treasury_call).unwrap();

		let alice_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_alice.clone());
		let bob_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_alice.clone());

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
				RuntimeEvent::Referenda(pallet_referenda::Event::Submitted { index: ref_index, .. }) => {},
			]
		);

		let decision_deposit =
			RuntimeCall::Referenda(pallet_referenda::Call::<Runtime>::place_decision_deposit { index: ref_index });

		assert_ok!(
			Referenda::place_decision_deposit(
				bob_origin.clone(),
				ref_index
			)
		);

		let addr = Into::<[u8; 32]>::into(polkadot_alice.clone());

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::Referenda(pallet_referenda::Event::DecisionDepositPlaced { index: ref_index, .. }) => {},
			]
		);

	});

}
