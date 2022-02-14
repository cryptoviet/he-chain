use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};

const POOL_FEE: u64 = 1_000_000;
const MARK_BLOCK: u64 = 3600;

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
			assert_eq!(balance_before, balance_after, "charge pool fee when fail not correct");
		}
	})
}

#[test]
fn should_move_newplayers_to_ingame() {
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
			let new_players_before = PalletPool::new_players();
			let ingame_players_before = PalletPool::ingame_players();

			assert_eq!(new_players_before.len(), 1, "new_players_before length not correct");
			assert_eq!(ingame_players_before.len(), 0, "ingame_players_before length not correct");
		}

		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			run_to_block(MARK_BLOCK + 1);
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after + POOL_FEE, "charge ingame players pool fee not correct");
		}

		{
			let new_players_after = PalletPool::new_players();
			let ingame_players_after = PalletPool::ingame_players();

			assert_eq!(new_players_after.len(), 0, "new_players_after length not correct");
			assert_eq!(ingame_players_after.len(), 1, "ingame_players_after length not correct");
		}
	})
}

