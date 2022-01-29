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

#[test]
fn player_leave_pool_should_works() {
    new_test_ext().execute_with(|| {
        let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000);
        assert_ok!(PalletPool::join(Origin::signed(ALICE)));
        assert_ok!(PalletPool::leave(Origin::signed(ALICE)));
    })
}

#[test]
fn player_leave_pool_should_fail() {
    new_test_ext().execute_with(|| {
        let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 1_000_000);
        let _ = <Test as Config>::Currency::deposit_creating(&BOB, 1_000_000);
        assert_err!(PalletPool::leave(Origin::signed(BOB)), <Error<Test>>::PlayerNotFound);
        assert_ok!(PalletPool::join(Origin::signed(ALICE)));
        assert_ok!(PalletPool::leave(Origin::signed(ALICE)));
        assert_err!(PalletPool::leave(Origin::signed(ALICE)), <Error<Test>>::PlayerNotFound);
    })
}

