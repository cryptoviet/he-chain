use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};

const POOL_FEE: u64 = 1_000_000;

#[test]
fn player_join_pool_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
		assert_ok!(PalletPool::join(Origin::signed(ALICE)));
		run_to_block(100);
		let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
		assert_eq!(balance_before, balance_after + POOL_FEE * 2, "charge pool fee not correct");
	});
}

#[test]
fn player_join_pool_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000);

		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_ok!(PalletPool::join(Origin::signed(ALICE)));
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after + POOL_FEE * 2, "charge pool fee not correct");
		}

		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_err!(PalletPool::join(Origin::signed(ALICE)), <Error<Test>>::PlayerAlreadyJoin);
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after,"charge pool fee when fail not correct");
		}
	})
}
