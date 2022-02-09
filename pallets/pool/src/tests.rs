use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};

const POOL_FEE: u64 = 1_000_000;

#[test]
fn player_join_pool_should_works() {
    ExtBuilder::default().build_and_execute(|| {
        run_to_block(10);
        let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
        assert_ok!(PalletPool::join(Origin::signed(ALICE)));
        let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
        assert_eq!(balance_before, balance_after + POOL_FEE);
    });
}

#[test]
fn player_join_pool_should_fail() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000);
        assert_ok!(PalletPool::join(Origin::signed(ALICE)));
        assert_err!(PalletPool::join(Origin::signed(ALICE)), <Error<Test>>::PlayerAlreadyJoin);

    })
}

