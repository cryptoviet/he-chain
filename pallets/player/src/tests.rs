use crate::{mock::*, Error};
use frame_support::{assert_ok, assert_err};

#[test]
fn create_new_player_should_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(PalletPlayer::new_player(Origin::signed(1), [0u8; 16]));
		
	});
}

#[test]
fn duplicated_create_player_should_fail() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(PalletPlayer::new_player(Origin::signed(1), [0u8; 16]));
		run_to_block(10);
		assert_err!(PalletPlayer::new_player(Origin::signed(1), [0u8; 16]), <Error::<Test>>::UsernameUsed);
	});
}

