use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};

#[test]
fn player_join_pool_should_works() {
    new_test_ext().execute_with(|| {
        let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1000_000);
        assert_ok!(PalletPool::join(Origin::signed(ALICE)));
    })
}

#[test]
fn player_join_pool_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000);
        assert_ok!(PalletPool::join(Origin::signed(ALICE)));
        assert_err!(PalletPool::join(Origin::signed(ALICE)), <Error<Test>>::PlayerAlreadyJoin);

    })
}

