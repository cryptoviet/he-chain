use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};

#[test]
fn host_game_should_available() {
	new_test_ext().execute_with(|| {
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 100_000_000);
		assert_ok!(PalletGame::is_host_available(&ALICE));
	});
}

#[test]
fn open_new_game_should_works() {
	new_test_ext().execute_with(|| {
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 100_000_000);
		assert_ok!(PalletGame::is_host_available(&ALICE));
		assert_ok!(PalletGame::open_game(ALICE, 10u64));
	});
}
