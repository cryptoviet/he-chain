use crate::{mock::*, Config, EndedGame, Error};
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
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[6][7] = player_index;
			game_map[8][7] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[5][7] = player_index;
			game_map[9][7] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[4][7] = player_index;
			game_map[10][7] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
		}

		{
			let mut game_map = [[-1i8; 15]; 15];
			let player_index: i8 = 1;
			let x: usize = 7;
			let y: usize = 7;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[7][6] = player_index;
			game_map[7][8] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[7][5] = player_index;
			game_map[7][9] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[7][4] = player_index;
			game_map[7][10] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
		}

		{
			let mut game_map = [[-1i8; 15]; 15];
			let player_index: i8 = 1;
			let x: usize = 7;
			let y: usize = 7;

			game_map[8][8] = player_index;
			game_map[6][6] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[9][9] = player_index;
			game_map[5][5] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[10][10] = player_index;
			game_map[4][4] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
		}

		{
			let mut game_map = [[-1i8; 15]; 15];
			let player_index: i8 = 1;
			let x: usize = 7;
			let y: usize = 7;

			game_map[6][8] = player_index;
			game_map[8][6] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, false, "should not win");

			game_map[5][9] = player_index;
			game_map[9][5] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");

			game_map[4][10] = player_index;
			game_map[10][4] = player_index;
			let is_winner = PalletGame::check_winner(game_map, player_index, x, y).unwrap();
			assert_eq!(is_winner, true, "should win");
		}
	});
}

#[test]
fn game_flow_should_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		let _ = <Test as Config>::Currency::deposit_creating(&ALICE, 100_000_000);
		let _ = <Test as Config>::Currency::deposit_creating(&BOB, 100_000_000);
		let ticket = 10_000;

		let alice_before_balance = <Test as Config>::Currency::free_balance(ALICE);
		let bob_before_balance = <Test as Config>::Currency::free_balance(BOB);

		assert_ok!(PalletGame::set_max_player(2u8));

		// Max game player
		let max_gomoku_player = PalletGame::max_gomoku_player();
		assert_eq!(max_gomoku_player, 2, "Gomoku max player not correct");
		// OPEN AND JOIN GAME
		assert_ok!(PalletGame::open_and_join(Origin::signed(ALICE), ticket));
		let game_open_ids = PalletGame::game_open();
		assert_eq!(game_open_ids.len(), 1, "game opened length not correct");
		let game_id = game_open_ids.first().unwrap();
		// check storage after open game
		{
			let game_open = PalletGame::game_open();
			assert_eq!(game_open.contains(game_id), true, "game_id should not on the GameOpen");

			let game_hosting = PalletGame::game_hosting(ALICE).unwrap();
			assert_eq!(game_hosting, *game_id, "Game hosting should exsit");
		}

		// JOIN GAME
		assert_ok!(PalletGame::join(Origin::signed(BOB), *game_id));

		let alice_after_balance = <Test as Config>::Currency::free_balance(ALICE);
		let bob_after_balance = <Test as Config>::Currency::free_balance(BOB);

		assert_eq!(alice_before_balance, alice_after_balance + ticket, "Alice balance not correct");
		assert_eq!(bob_before_balance, bob_after_balance + ticket, "Bob balance not correct");

		// START GAME
		assert_ok!(PalletGame::start(Origin::signed(BOB)));
		run_to_block(12);

		// check storage after start game
		{
			let game_open = PalletGame::game_open();
			assert_eq!(game_open.contains(game_id), false, "game_id should not on the GameOpen");
			let game_start = PalletGame::game_start();
			assert_eq!(game_start.contains(game_id), true, "game_id should on the GameStart");
		}

		// PLAY GAME
		{
			// turn 1
			assert_err!(PalletGame::play(Origin::signed(ALICE), 8, 8), <Error<Test>>::NotYourTurn);
			assert_ok!(PalletGame::play(Origin::signed(BOB), 7, 7));

			// //turn 2
			assert_err!(PalletGame::play(Origin::signed(BOB), 8, 8), <Error<Test>>::NotYourTurn);
			assert_ok!(PalletGame::play(Origin::signed(ALICE), 1, 1));

			// // turn 3
			assert_ok!(PalletGame::play(Origin::signed(BOB), 7, 6));
			assert_ok!(PalletGame::play(Origin::signed(ALICE), 1, 2));
			assert_ok!(PalletGame::play(Origin::signed(BOB), 7, 5));
			assert_ok!(PalletGame::play(Origin::signed(ALICE), 1, 3));
			assert_ok!(PalletGame::play(Origin::signed(BOB), 7, 4));
			assert_ok!(PalletGame::play(Origin::signed(ALICE), 1, 4));

			// BOB win this game
			{
				let bob_before_balance = <Test as Config>::Currency::free_balance(BOB);
				assert_ok!(PalletGame::play(Origin::signed(BOB), 7, 3));
				let bob_after_balance = <Test as Config>::Currency::free_balance(BOB);

				let mut reward = ticket * 2;
				reward = (reward as f64 - (reward as f64 * 0.01)) as u64;
				assert_eq!(bob_after_balance - bob_before_balance, reward, "reward receipt not correct");
			}

			run_to_block(20);
			assert_err!(
				PalletGame::play(Origin::signed(ALICE), 1, 5),
				<Error<Test>>::PlayerNotPlaying
			);
			assert_err!(
				PalletGame::play(Origin::signed(BOB), 1, 5),
				<Error<Test>>::PlayerNotPlaying
			);

			// check storage
			{
				let ended_game: EndedGame<Test> = PalletGame::ended_game(game_id).unwrap();
				assert_eq!(ended_game.winner, BOB, "winner not correct");

				let ended_games = PalletGame::get_ended_games();
				assert_eq!(ended_games.contains(game_id), true, "ended_games must contain game_id");
			}
		}
	});
}
