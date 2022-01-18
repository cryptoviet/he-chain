#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{print, traits::Hash},
		traits::{
			tokens::{ExistenceRequirement, WithdrawReasons},
			Currency, Randomness,
		},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_256;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type ID = [u8; 32];

	// ACTION #1: Write a Struct to hold Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Game<T: Config> {
		id: ID,
		host: T::AccountId,
		ticket: BalanceOf<T>,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct EndedGame<T: Config> {
		id: ID,
		host: T::AccountId,
		ticket: BalanceOf<T>,
		game_map: [[i8; 15]; 15],
		winner: T::AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum GameStatus {
		Open,
		Start,
		End,
		Cancel,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		type GameRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type MaxGomokuPlayer: Get<u32>;

		#[pallet::constant]
		type MaxGame: Get<u32>;

		#[pallet::constant]
		type MaxOpenGame: Get<u32>;

		#[pallet::constant]
		type MaxStartGame: Get<u32>;

		#[pallet::constant]
		type OpenGameFee: Get<u32>;

		#[pallet::constant]
		type MaxEndedGame: Get<u32>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		GameNotExist,
		GameNotOpen,
		GameIdUsed,
		GameOverflow,
		GameOpenNotFound,
		GamePlayingNotFound,
		GameEndedNotFound,
		ExceedGameOpen,
		PlayerExceed,
		PlayersOverflow,
		PlayersNotFound,
		PlayerNotPlaying,
		YouAreInGame,
		ExceedGameHosting,

		//Start
		NotEnoughPlayer,
		NotYourTurn,
		GameMapNotFound,
		PlaceNotEmpty,
	}

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewGameOpen(ID, T::AccountId, BalanceOf<T>),
		PlayerJoinGame(T::AccountId, ID),
	}

	#[pallet::storage]
	#[pallet::getter(fn game_cnt)]
	pub(super) type GameCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	// ACTION #7: Remaining storage items.

	#[pallet::storage]
	#[pallet::getter(fn open_game_fee)]
	pub(super) type OpenGameFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn max_gomoku_player)]
	pub(super) type MaxGomoku<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn game_open)]
	pub(super) type GameOpen<T: Config> =
		StorageValue<_, BoundedVec<ID, T::MaxOpenGame>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn game_start)]
	pub(super) type GameStart<T: Config> =
		StorageValue<_, BoundedVec<ID, T::MaxStartGame>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn games)]
	pub(super) type Games<T: Config> = StorageMap<_, Twox64Concat, ID, Game<T>>;
	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> =
		StorageMap<_, Twox64Concat, ID, BoundedVec<T::AccountId, T::MaxGomokuPlayer>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn game_playing)]
	pub(super) type GamePlaying<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ID>;

	#[pallet::storage]
	#[pallet::getter(fn game_hosting)]
	pub(super) type GameHosting<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ID>;

	#[pallet::storage]
	#[pallet::getter(fn ended_game)]
	pub(super) type EndedGames<T: Config> = StorageMap<_, Twox64Concat, ID, EndedGame<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_ended_games)]
	pub(super) type GetEndedGames<T: Config> =
		StorageValue<_, BoundedVec<ID, T::MaxEndedGame>, ValueQuery>;

	// GAME LOGIC STORAGE
	#[pallet::storage]
	#[pallet::getter(fn gomoku_game)]
	pub(super) type GomokuGame<T: Config> = StorageMap<_, Twox64Concat, ID, [[i8; 15]; 15]>;

	#[pallet::storage]
	#[pallet::getter(fn turn)]
	pub(super) type Turn<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub open_fee: BalanceOf<T>,
		pub max_gomoku_player: u8,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { open_fee: Default::default(), max_gomoku_player: 2u8 }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<OpenGameFee<T>>::put(self.open_fee);
			<MaxGomoku<T>>::put(self.max_gomoku_player);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn open(origin: OriginFor<T>, ticket: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let id = Self::open_game(sender.clone(), ticket)?;
			Self::charge_fee_open_game(&sender)?;
			Self::deposit_event(Event::NewGameOpen(id, sender, ticket));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn join(origin: OriginFor<T>, game_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let game = Self::join_game(sender.clone(), &game_id)?;
			Self::charge_join_game(&sender, game.ticket)?;
			Self::deposit_event(Event::PlayerJoinGame(sender, game_id));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn open_and_join(origin: OriginFor<T>, ticket: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let id = Self::open_game(sender.clone(), ticket)?;
			Self::charge_fee_open_game(&sender)?;
			Self::deposit_event(Event::NewGameOpen(id, sender.clone(), ticket));

			let game = Self::join_game(sender.clone(), &id)?;
			Self::charge_join_game(&sender, game.ticket)?;
			Self::deposit_event(Event::PlayerJoinGame(sender, id));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn left(origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn start(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::start_game(&sender)?;
			Ok(())
		}

		#[pallet::weight(1)]
		pub fn play(origin: OriginFor<T>, x: u32, y: u32) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::play_game(&sender, x as usize, y as usize)?;
			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {
		#[transactional]
		pub fn open_game(sender: T::AccountId, ticket: BalanceOf<T>) -> Result<ID, Error<T>> {
			Self::is_host_available(&sender)?;
			let new_game_cnt = Self::game_cnt().checked_add(1).ok_or(<Error<T>>::GameOverflow)?;
			<GameCnt<T>>::put(new_game_cnt);
			let id = Self::gen_id()?;
			Self::is_id_available(id)?;
			let game = Game::<T> { id, host: sender.clone(), ticket };

			<GameOpen<T>>::try_mutate(|game_open| game_open.try_push(id))
				.map_err(|_| <Error<T>>::ExceedGameOpen)?;
			<Games<T>>::insert(id, game);
			<GameHosting<T>>::insert(sender, id);
			Ok(id)
		}

		#[transactional]
		pub fn start_game(sender: &T::AccountId) -> Result<(), Error<T>> {
			let id_game_playing = Self::get_game_playing(sender)?;
			let players = Self::players(id_game_playing);

			ensure!(players.len() as u8 == Self::max_gomoku_player(), <Error<T>>::NotEnoughPlayer);

			<GomokuGame<T>>::insert(id_game_playing, [[-1i8; 15]; 15]);

			<GameOpen<T>>::try_mutate(|id_vec| {
				if let Some(ind) = id_vec.iter().position(|&id| id == id_game_playing) {
					id_vec.swap_remove(ind);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::GameOpenNotFound)?;

			<GameStart<T>>::try_mutate(|game_start| game_start.try_push(id_game_playing))
				.map_err(|_| <Error<T>>::GameNotExist)?;

			<Turn<T>>::put(sender);
			Ok(())
		}

		#[transactional]
		pub fn play_game(sender: &T::AccountId, x: usize, y: usize) -> Result<(), Error<T>> {
			ensure!(sender == &Self::turn(), <Error<T>>::NotYourTurn);
			let game_playing_id = Self::get_game_playing(sender)?;
			let player_index = Self::get_player_index(&game_playing_id, &sender)?;
			ensure!(
				(Self::gomoku_game(game_playing_id).unwrap())[x][y] == -1i8,
				<Error<T>>::PlaceNotEmpty
			);

			// check winner
			let gomoku_game = Self::gomoku_game(game_playing_id).unwrap();
			let game_result = Self::check_winner(&gomoku_game, player_index, x, y)?;

			if game_result == false {
				Self::continue_game(sender, &game_playing_id, x, y, player_index)?;
			} else {
				Self::finish_game(sender.clone(), game_playing_id, gomoku_game)?;
			}
			Ok(())
		}

		pub fn continue_game(
			sender: &T::AccountId,
			game_id: &ID,
			x: usize,
			y: usize,
			player_index: i8,
		) -> Result<(), Error<T>> {
			<GomokuGame<T>>::try_mutate(game_id, |gomoku| {
				let mut new_gomoku = gomoku.unwrap();
				new_gomoku[x][y] = player_index;
				*gomoku = Some(new_gomoku);
				Ok(())
			})
			.map_err(|_: Error<T>| <Error<T>>::GameMapNotFound)?;

			let other_player = Self::get_other_player(&game_id, sender)?;
			<Turn<T>>::put(other_player);
			Ok(())
		}

		pub fn finish_game(
			winner: T::AccountId,
			game_id: ID,
			game_map: [[i8; 15]; 15],
		) -> Result<(), Error<T>> {
			let game = Self::get_game(&game_id)?;

			<GamePlaying<T>>::remove(winner.clone());
			<Players<T>>::remove(game_id);
			<GameHosting<T>>::remove(game.host.clone());

			let ended_game =
				EndedGame { id: game.id, host: game.host, ticket: game.ticket, winner: winner.clone(), game_map };

			<GetEndedGames<T>>::try_mutate(|game_vec| game_vec.try_push(game_id))
				.map_err(|_| <Error<T>>::GameEndedNotFound)?;

			<EndedGames<T>>::insert(game_id, ended_game);
			let ticket = Self::balance_to_u64(game.ticket).unwrap();
			let ticket = Self::u64_to_balance((ticket as f64 - (ticket as f64 * 0.01)) as u64).unwrap();
			let _ = T::Currency::deposit_into_existing(&winner, ticket);
			Ok(())
		}

		#[transactional]
		pub fn join_game(sender: T::AccountId, game_id: &ID) -> Result<Game<T>, Error<T>> {
			// make sure game id exsit
			let game = Self::get_game(game_id)?;
			Self::is_game_open(game_id)?;

			// make sute player not playing
			Self::is_player_available(&sender)?;
			Self::player_join_game(sender, game_id)?;
			Ok(game)
		}

		pub fn player_join_game(sender: T::AccountId, game_id: &ID) -> Result<(), Error<T>> {
			let players = <Players<T>>::get(game_id);
			ensure!((players.len() as u8) < Self::max_gomoku_player(), <Error<T>>::PlayerExceed);
			<Players<T>>::try_mutate(game_id, |player_vec| player_vec.try_push(sender.clone()))
				.map_err(|_| <Error<T>>::PlayersOverflow)?;
			<GamePlaying<T>>::insert(sender, game_id);
			Ok(())
		}

		pub fn player_left_game(sender: T::AccountId, game: &Game<T>) -> Result<(), Error<T>> {
			let game_id = Self::get_game_playing(&sender)?;
			let game = Self::get_game(&game_id)?;

			Ok(())
		}

		#[transactional]
		pub fn left_game(sender: T::AccountId) -> Result<(), Error<T>> {
			let game_id = Self::get_game_playing(&sender)?;
			let game = Self::get_game(&game_id)?;

			// make sure game status is open

			let players = <Players<T>>::get(game_id);

			Ok(())
		}

		pub fn charge_join_game(sender: &T::AccountId, ticket: BalanceOf<T>) -> DispatchResult {
			let withdraw = T::Currency::withdraw(
				&sender,
				ticket,
				WithdrawReasons::RESERVE,
				ExistenceRequirement::KeepAlive,
			);

			match withdraw {
				Ok(_) => Ok(()),
				Err(err) => Err(err),
			}
		}

		pub fn charge_fee_open_game(sender: &T::AccountId) -> DispatchResult {
			let open_game_fee = Self::open_game_fee();
			let withdraw = T::Currency::withdraw(
				&sender,
				open_game_fee,
				WithdrawReasons::RESERVE,
				ExistenceRequirement::KeepAlive,
			);

			match withdraw {
				Ok(_) => Ok(()),
				Err(err) => Err(err),
			}
		}

		pub fn gen_id() -> Result<ID, Error<T>> {
			let payload =
				(T::GameRandomness::random(&b""[..]).0, <frame_system::Pallet<T>>::block_number());
			Ok(payload.using_encoded(blake2_256))
		}

		pub fn is_id_available(id: ID) -> Result<bool, Error<T>> {
			match Self::games(id) {
				Some(_) => Err(<Error<T>>::GameIdUsed),
				None => Ok(true),
			}
		}

		pub fn get_game(id: &ID) -> Result<Game<T>, Error<T>> {
			match Self::games(id) {
				Some(game) => Ok(game),
				None => Err(<Error<T>>::GameNotExist),
			}
		}

		pub fn is_player_available(player: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::game_playing(player) {
				Some(_) => Err(<Error<T>>::YouAreInGame),
				None => Ok(true),
			}
		}

		pub fn is_host_available(player: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::game_hosting(player) {
				Some(_) => Err(<Error<T>>::ExceedGameHosting),
				None => Ok(true),
			}
		}

		pub fn get_game_playing(player: &T::AccountId) -> Result<ID, Error<T>> {
			match Self::game_playing(player) {
				Some(id) => Ok(id),
				None => Err(<Error<T>>::PlayerNotPlaying),
			}
		}

		pub fn is_game_open(game_id: &ID) -> Result<bool, Error<T>> {
			match <GameOpen<T>>::get().binary_search(game_id) {
				Ok(_) => Ok(true),
				Err(_) => Err(<Error<T>>::GameOpenNotFound),
			}
		}

		pub fn get_player_index(game_id: &ID, player: &T::AccountId) -> Result<i8, Error<T>> {
			let slice = Self::players(game_id);
			let index = slice.iter().position(|p| p == player).unwrap();
			Ok(index as i8)
		}

		pub fn get_other_player(
			game_id: &ID,
			player: &T::AccountId,
		) -> Result<T::AccountId, Error<T>> {
			let slice = Self::players(game_id);
			let other_player = slice.iter().position(|p| p != player).unwrap();
			let other = &slice[other_player];
			Ok(other.clone())
		}

		pub fn check_winner(
			game_map: &[[i8; 15]; 15],
			player_index: i8,
			x: usize,
			y: usize,
		) -> Result<bool, Error<T>> {
			// check horizontal
			let horizontal_check = || -> bool {
				for index in 1..3 {
					if game_map[x + index][y] != player_index {
						return false
					}
					if game_map[x - index][y] != player_index {
						return false
					}
				}
				return true
			};

			let vertical_check = || -> bool {
				for index in 1..3 {
					if game_map[x][y + index] != player_index {
						return false
					}
					if game_map[x][y - index] != player_index {
						return false
					}
				}
				return true
			};

			let topright_bottomleft_check = || -> bool {
				for index in 1..3 {
					if game_map[x + index][y + index] != player_index {
						return false
					}
					if game_map[x - index][y - index] != player_index {
						return false
					}
				}
				return true
			};

			let topleft_bottomright_check = || -> bool {
				for index in 1..3 {
					if game_map[x - index][y + index] != player_index {
						return false
					}
					if game_map[x + index][y - index] != player_index {
						return false
					}
				}
				return true
			};

			if horizontal_check() ||
				vertical_check() || topright_bottomleft_check() ||
				topleft_bottomright_check()
			{
				Ok(true)
			} else {
				Ok(false)
			}
		}

		pub fn balance_to_u64(balance: BalanceOf<T>) -> Option<u64> {
			TryInto::<u64>::try_into(balance).ok()
		}

		pub fn u64_to_balance(num: u64) -> Option<BalanceOf<T>> {
			num.try_into().ok()
		}
	}
}
