#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	pallet_prelude::*,
	traits::{
		tokens::{ExistenceRequirement, WithdrawReasons},
		Currency, Randomness,
	},
};

use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize,
		Saturating, StaticLookup, Zero,
	},
	ArithmeticError, DispatchError, RuntimeDebug,
};

use frame_system::pallet_prelude::*;
use sp_io::hashing::blake2_128;

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
pub use pallet::*;
pub use pallet_player::PlayerOwned;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type ID = [u8; 32];

	// Struct, Enum
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Player<T: Config> {
		address: T::AccountId,
		join_block: T::BlockNumber,
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

		#[pallet::constant]
		type MaxNewPlayer: Get<u32>;

		#[pallet::constant]
		type MaxIngamePlayer: Get<u32>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		PlayerNotFound,
		PlayerAlreadyJoin,

		PlayerCountOverflow,
		ExceedMaxPlayer,
		ExceedMaxNewPlayer,
		CanNotClearNewPlayers,
		ExceedMaxIngamePlayer,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PlayerJoinPool(T::AccountId),
		PlayerLeavePool(T::AccountId),
	}

	/*
		1. Charge player in the IngamePlayers
			1.1 Kick player when they can't pay
		2. Move all players from NewPlayer to IngamePlayers
	*/
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			let mark_block = Self::mark_block();

			if (block_number % mark_block.into()).is_zero() {
				let _ = Self::charge_ingame();

				let _ = Self::move_newplayer_to_ingame();
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn max_player)]
	pub type MaxPlayer<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mark_block)]
	pub type MarkBlock<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_fee)]
	pub type PoolFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Player<T>>;

	#[pallet::storage]
	#[pallet::getter(fn player_count)]
	pub(super) type PlayerCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn new_players)]
	pub(super) type NewPlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxNewPlayer>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn ingame_players)]
	pub type IngamePlayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxIngamePlayer>, ValueQuery>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub max_player: u32,
		pub mark_block: u32,
		pub pool_fee: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { max_player: 1000, mark_block: 1200, pool_fee: 1000000u32.into() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<MaxPlayer<T>>::put(self.max_player);
			<MarkBlock<T>>::put(self.mark_block);
			<PoolFee<T>>::put(self.pool_fee);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn join(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::join_pool(sender.clone())?;
			let pool_fee = Self::pool_fee();
			let double_fee = pool_fee * 2u32.into();
			Self::change_fee(&sender, double_fee)?;
			Self::deposit_event(Event::PlayerJoinPool(sender));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn leave(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::leave_pool(&sender)?;
			Self::deposit_event(Event::PlayerLeavePool(sender));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn join_pool(sender: T::AccountId) -> Result<(), Error<T>> {
			// make sure player not re-join
			ensure!(Self::players(sender.clone()) == None, <Error<T>>::PlayerAlreadyJoin);
			// make sure not exceed max players
			let new_player_count =
				Self::player_count().checked_add(1).ok_or(<Error<T>>::PlayerCountOverflow)?;
			ensure!(new_player_count <= Self::max_player(), <Error<T>>::ExceedMaxPlayer);
			// make sure not exceed max new players
			<NewPlayers<T>>::try_mutate(|newplayers| newplayers.try_push(sender.clone()))
				.map_err(|_| <Error<T>>::ExceedMaxNewPlayer)?;
			let block_number = <frame_system::Pallet<T>>::block_number();
			let player = Player::<T> { address: sender.clone(), join_block: block_number };
			<Players<T>>::insert(sender, player);
			<PlayerCount<T>>::put(new_player_count);
			Ok(())
		}

		pub fn change_fee(sender: &T::AccountId, fee: BalanceOf<T>) -> DispatchResult {
			let withdraw = T::Currency::withdraw(
				&sender,
				fee,
				WithdrawReasons::RESERVE,
				ExistenceRequirement::KeepAlive,
			);

			match withdraw {
				Ok(_) => Ok(()),
				Err(err) => Err(err),
			}
		}

		fn leave_pool(sender: &T::AccountId) -> Result<(), Error<T>> {
			Ok(())
		}

		/*
			Remove player from storage Players and PlayerOwned
		*/
		fn kick_player(player: &T::AccountId) -> Result<(), Error<T>> {
			Ok(())
		}

		fn charge_ingame() -> Result<(), Error<T>> {
			let ingame_players: Vec<T::AccountId> = Self::new_players().into_inner();
			for player in ingame_players {
				match Self::change_fee(&player, Self::pool_fee()) {
					Ok(_) => {},
					Err(_) => {},
				}
			}
			Ok(())
		}


		fn move_newplayer_to_ingame() -> Result<(), Error<T>> {
			let new_players: Vec<T::AccountId> = Self::new_players().into_inner();
			for new_player in new_players {
				<IngamePlayers<T>>::try_append(new_player)
					.map_err(|_| <Error<T>>::ExceedMaxNewPlayer)?;
			}

			<NewPlayers<T>>::kill();

			Ok(())
		}
	}
}

#[cfg(feature = "std")]
impl<T: Config> GenesisConfig<T> {
	pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
		<Self as GenesisBuild<T>>::build_storage(self)
	}

	pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
		<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
	}
}
