#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		traits::{tokens::ExistenceRequirement, tokens::WithdrawReasons, Currency, Randomness},
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
		number_of_player: u8,
		ticket: BalanceOf<T>,
		status: GameStatus,
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

		/// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId>;

		type GameRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type MaxPlayer: Get<u32>;

		#[pallet::constant]
		type MaxGame: Get<u32>;

		#[pallet::constant]
		type MaxOpenGame: Get<u32>;

		#[pallet::constant]
		type MaxStartGame: Get<u32>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		GameNotExist,
		GameNotOpen,
		GameIdUsed,
		ExceedGameOpen,
		GameOverflow,
		PlayerExceed,
		PlayersOverflow,
		PlayersNotFound,
		YouAreInGame,
	}

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewGameOpen(ID, T::AccountId, u8, BalanceOf<T>),
		PlayerJoinGame(T::AccountId, ID),
	}

	#[pallet::storage]
	#[pallet::getter(fn game_cnt)]
	pub(super) type GameCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	// ACTION #7: Remaining storage items.

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
		StorageMap<_, Twox64Concat, ID, BoundedVec<T::AccountId, T::MaxPlayer>, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn game_playing)]
	pub(super) type GamePlaying <T: Config> = 
		StorageMap<_, Twox64Concat, T::AccountId, ID>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn open(
			origin: OriginFor<T>,
			number_of_player: u8,
			ticket: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let id = Self::open_game(sender.clone(), number_of_player, ticket)?;
			T::Currency::withdraw(&sender, ticket, WithdrawReasons::RESERVE, ExistenceRequirement::KeepAlive)?;
			Self::deposit_event(Event::NewGameOpen(id, sender, number_of_player, ticket));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn join(origin: OriginFor<T>, game_id: ID) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let game = Self::join_game(sender.clone(), &game_id)?;
			T::Currency::withdraw(&sender, game.ticket, WithdrawReasons::RESERVE, ExistenceRequirement::KeepAlive)?;
			Self::deposit_event(Event::PlayerJoinGame(sender, game_id));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn left(origin: OriginFor<T>) -> DispatchResult {

			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {

		#[transactional]
		pub fn open_game(
			sender: T::AccountId,
			number_of_player: u8,
			ticket: BalanceOf<T>,
		) -> Result<ID, Error<T>> {

			Self::is_player_available(&sender)?;
			
			let new_game_cnt = Self::game_cnt().checked_add(1).ok_or(<Error<T>>::GameOverflow)?;
			<GameCnt<T>>::put(new_game_cnt);
			
			let id = Self::gen_id()?;
			Self::is_id_available(id)?;
			let game =
			Game::<T> { id, host: sender.clone(), number_of_player, ticket, status: GameStatus::Open };
			<GameOpen<T>>::try_mutate(|game_open| game_open.try_push(id))
			.map_err(|_| <Error<T>>::ExceedGameOpen)?;
			<GamePlaying<T>>::insert(sender, id);
			<Games<T>>::insert(id, game);
			Ok(id)
		}

		#[transactional]
		pub fn join_game(sender: T::AccountId, game_id: &ID) -> Result<Game<T>, Error<T>>{

			// make sure game id exsit
			let game = Self::get_game(game_id)?;
			ensure!(game.status == GameStatus::Open, <Error<T>>::GameNotOpen);

			// make sute player not playing
			Self::is_player_available(&sender)?;

			// make sure number allowed is good
			let players = <Players<T>>::get(game_id);
			ensure!(players.len() < game.number_of_player.into(), <Error<T>>::PlayerExceed);

			<Players<T>>::try_mutate(game_id, |player_vec| {
					player_vec.try_push(sender.clone())
			}).map_err(|_| <Error<T>>::PlayersOverflow)?;
			<GamePlaying<T>>::insert(sender, game_id);
			Ok(game)
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
				None => Err(<Error<T>>::GameIdUsed),
			}
		}

		pub fn is_player_available(player: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::game_playing(player) {
				Some(_) => Err(<Error<T>>::YouAreInGame),
				None => Ok(true),
			}
		}
		
	}
}
