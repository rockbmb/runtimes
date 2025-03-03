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

use frame_support::{
	sp_runtime::traits::Dispatchable,
	traits::{schedule::DispatchTime, StorePreimage}
};

use pallet_conviction_voting::Tally;
use pallet_referenda::{ReferendumInfoFor, TallyOf};

use polkadot_runtime::{
	governance::{
		pallet_custom_origins::{self, Origin},
		ReferendumCanceller,
		ReferendumKiller
	}, ConvictionVoting, Preimage, Referenda, RuntimeCall, RuntimeOrigin
};
use polkadot_runtime_common::impls::VersionedLocatableAsset;

use polkadot_runtime_constants::currency::UNITS as DOT;

use frame_system::Pallet as System;

#[test]
fn referendum_lifecycle() {
	let polkadot_alice = Polkadot::account_id_of(ALICE);

	let ref_origins = vec![
		Origin::StakingAdmin,
		Origin::Treasurer,
		Origin::FellowshipAdmin,
		Origin::GeneralAdmin,
		Origin::AuctionAdmin,
		Origin::LeaseAdmin,
		Origin::ReferendumCanceller,
		Origin::ReferendumKiller,
		Origin::SmallTipper,
		Origin::BigTipper,
		Origin::SmallSpender,
		Origin::MediumSpender,
		Origin::BigSpender,
		Origin::WhitelistedCaller,
		Origin::WishForChange,
	];
	let origins: Vec<(OriginKind, RuntimeOrigin)> = vec![
		//(OriginKind::Native, Origin::ReferendumCanceller.into()),
		(OriginKind::Superuser, <Polkadot as Chain>::RuntimeOrigin::root()),
	];

	let mut ref_index: u32 = 0;

	for (origin_kind, origin) in origins {
		for ref_origin in &ref_origins {
			Polkadot::execute_with(|| {				
				type Runtime = <Polkadot as Chain>::Runtime;
				type RuntimeCall = <Polkadot as Chain>::RuntimeCall;
				type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;
				type Referenda = <Polkadot as PolkadotPallet>::Referenda;

				let alice_origin = <Polkadot as Chain>::RuntimeOrigin::signed(polkadot_alice.clone());

				// Create proposal on which the referendum will be based
				let call = RuntimeCall::System(frame_system::Call::<Runtime>::remark {
					remark: vec![42, 42, 42]
				});

				let proposal = <Preimage as StorePreimage>::bound(call).unwrap();

				// Submit referendum
				assert_ok!(
					Referenda::submit(
						alice_origin.clone(),
						Box::new(Origin::SmallTipper.into()),
						proposal,
						DispatchTime::At(0u32.into())
					)
				);

				assert_expected_events!(
					Polkadot,
					vec![
						RuntimeEvent::Referenda(pallet_referenda::Event::Submitted { index: ref_index, .. }) => {},
					]
				);

/* 				// Cancel referendum
				assert_ok!(
					Referenda::cancel(
						origin.clone(),
						ref_index
					)
				);

				assert_expected_events!(
					Polkadot,
					vec![
						RuntimeEvent::Referenda(pallet_referenda::Event::Cancelled {
							index: ref_index,
							tally:
							Tally {
								ayes: 0,
								nays: 0,
								support: 0,
								..
							} 
						}) => {},
					]
				);
*/

				let cancel_ref_call =
				RuntimeCall::Referenda(pallet_referenda::Call::<Runtime>::cancel { index: ref_index });

				let xcm_message = RuntimeCall::XcmPallet(pallet_xcm::Call::<Runtime>::send {
					dest: bx!(VersionedLocation::from(Location::new(0, Here))),
					message: bx!(VersionedXcm::from(Xcm(vec![
						UnpaidExecution { weight_limit: Unlimited, check_origin: None },
						Transact {
							origin_kind,
							require_weight_at_most: Weight::from_parts(100_000_000_000, 1_000_000),
							call: cancel_ref_call.encode().into(),
						}
					]))),
				});
		
				assert_ok!(xcm_message.dispatch(origin.clone()));
		
				assert_expected_events!(
					Polkadot,
					vec![
						RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
					]
				);

			});

			ref_index += 1;
		}
	}
}
