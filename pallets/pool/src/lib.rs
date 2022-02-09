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
		sp_runtime::{
			traits::{Hash, Zero},
		},
		traits::{
			tokens::{ExistenceRequirement, WithdrawReasons},
			Currency, Randomness,
		},
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type ID = [u8; 32];

	// Struct, Enum
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Player<T: Config> {
		address: T::AccountId,
		start_block: T::BlockNumber,
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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {}
	}

	#[pallet::storage]
	#[pallet::getter(fn mark_block)]
	pub type MarkBlock<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_fee)]
	pub type PoolFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Player<T>>;

	//** Genesis Conguration **//
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub mark_block: u32,
		pub pool_fee: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { mark_block: 3600, pool_fee: 1000000u32.into() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<MarkBlock<T>>::put(self.mark_block);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn join(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::join_pool(sender.clone())?;
			Self::charge_join_pool(&sender)?;
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
			ensure!(Self::players(sender.clone()) == None, <Error<T>>::PlayerAlreadyJoin);
			let block_number = <frame_system::Pallet<T>>::block_number();
			let player = Player::<T> { address: sender.clone(), start_block: block_number };
			<Players<T>>::insert(sender, player);
			Ok(())
		}

		pub fn charge_join_pool(sender: &T::AccountId) -> DispatchResult {
			let pool_fee = Self::pool_fee();

			println!("fee: {:?}", pool_fee);

			let withdraw = T::Currency::withdraw(
				&sender,
				pool_fee,
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
	}
}
