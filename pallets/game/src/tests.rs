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
		run_to_block(10);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 100_000_000);
		assert_ok!(PalletGame::is_host_available(&ALICE));
		assert_ok!(PalletGame::open_game(ALICE, 10u64));
	});
}

#[test]
fn test_winner_should_works() {
	new_test_ext().execute_with(|| {
		{
			let mut game_map = [[-1i8; 15]; 15];
			let player_index: i8 = 1;
			let x: usize = 7;
			let y: usize = 7;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[6][7] = player_index;
			game_map[8][7] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[5][7] = player_index;
			game_map[9][7] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[4][7] = player_index;
			game_map[10][7] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
		}

		{
            let mut game_map = [[-1i8; 15]; 15];
			let player_index: i8 = 1;
			let x: usize = 7;
			let y: usize = 7;
            let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[7][6] = player_index;
			game_map[7][8] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[7][5] = player_index;
			game_map[7][9] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[7][4] = player_index;
			game_map[7][10] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
        }

        {
            let mut game_map = [[-1i8; 15]; 15];
			let player_index: i8 = 1;
			let x: usize = 7;
			let y: usize = 7;

			game_map[8][8] = player_index;
			game_map[6][6] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[9][9] = player_index;
			game_map[5][5] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[10][10] = player_index;
			game_map[4][4] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
        }

        {
            let mut game_map = [[-1i8; 15]; 15];
			let player_index: i8 = 1;
			let x: usize = 7;
			let y: usize = 7;

			game_map[6][8] = player_index;
			game_map[8][6] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[5][9] = player_index;
			game_map[9][5] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[4][10] = player_index;
			game_map[10][4] = player_index;
			let is_winner = PalletGame::check_winner(&game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
        }


	});
}
