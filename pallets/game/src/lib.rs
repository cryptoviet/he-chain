#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

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

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	type ID = [u8; 32];
	type NAME = [u8; 16];

	// Struct, Enum
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Player<T: Config> {
		id: ID,
		owner: T::AccountId,
		name: NAME,
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
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		PlayerIdUsed,
		PlayerExisted,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewPlayerCreated(T::AccountId, ID),
	}

	// Storage
	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, ID, Player<T>>;

	#[pallet::storage]
	#[pallet::getter(fn player_owned)]
	pub(super) type PlayerOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, ID>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn create_player(origin: OriginFor<T>, name: NAME) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let id = Self::create_new_player(sender.clone(), name)?;
			Self::deposit_event(Event::NewPlayerCreated(sender, id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn gen_id() -> Result<ID, Error<T>> {
			let payload = (
				T::GameRandomness::random(&b""[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			Ok(payload.using_encoded(blake2_256))
		}

		pub fn create_new_player(sender: T::AccountId, user_name: NAME) -> Result<ID, Error<T>> {
			Self::is_player_available(&sender)?;
			let id = Self::gen_id()?;
			Self::is_player_id_available(&id)?;
			let player = Player::<T> { id, owner: sender.clone(), name: user_name };

			<Players<T>>::insert(id, player);
			<PlayerOwned<T>>::insert(sender, id);
			Ok(id)
		}

		pub fn is_player_id_available(id: &ID) -> Result<bool, Error<T>> {
			match Self::players(id) {
				Some(_) => Err(<Error<T>>::PlayerIdUsed),
				None => Ok(true),
			}
		}

		pub fn is_player_available(player: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::player_owned(player) {
				Some(_) => Err(<Error<T>>::PlayerExisted),
				None => Ok(true),
			}
		}
	}
}
