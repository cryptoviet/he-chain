#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use pallet_player::PlayerOwned;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		sp_runtime::traits::{Hash, Zero},
		traits::{Currency, ExistenceRequirement, Randomness},
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	type ID = [u8; 32];

	// Struct, Enum

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
		type MaxPoolPlayer: Get<u32>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		ExceedPoolPlayer,
		PlayerNotFound,
		PlayerAlreadyJoin,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PlayerJoinPool(T::AccountId),
		PlayerLeavePool(T::AccountId),
	}

	// Storage
	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxPoolPlayer>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn join(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::join_pool(sender.clone())?;
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
			ensure!(Self::get_player_index(&sender) == None, <Error<T>>::PlayerAlreadyJoin);
			<Players<T>>::try_mutate(|player_vec| player_vec.try_push(sender))
				.map_err(|_| <Error<T>>::ExceedPoolPlayer)?;
			Ok(())
		}

		fn leave_pool(sender: &T::AccountId) -> Result<(), Error<T>> {
			<Players<T>>::try_mutate(|player_vec| {
				if let Some(ind) = player_vec.iter().position(|player| player == sender) {
					player_vec.swap_remove(ind);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::PlayerNotFound)?;
			Ok(())
		}

		fn get_player_index(player: &T::AccountId) -> Option<usize> {
			let players = Self::players();
			players.iter().position(|p| p == player)
		}

	}
}
