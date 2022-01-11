#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		traits::{tokens::ExistenceRequirement, Currency, Randomness},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// ACTION #1: Write a Struct to hold Kitty information.
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Player<T: Config> {
		pub owner: AccountOf<T>,
		pub username: [u8; 16],
        pub level: u32,
        pub exp: u32,
	}

	// ACTION #2: Enum declaration for Gender.

	// ACTION #3: Implementation to handle Gender type in Kitty struct.

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
        

		// ACTION #5: Specify the type for Randomness we want to specify for runtime.

		// ACTION #9: Add MaxKittyOwned constant
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		UsernameUsed,
		PlayerOverflow,
	}

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PlayerCreated(T::AccountId, [u8;16])
	}

	#[pallet::storage]
	#[pallet::getter(fn player_cnt)]
	pub(super) type PlayerCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn players)]
    pub(super) type Players<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Player<T>>;

	#[pallet::storage]
	#[pallet::getter(fn username)]
	pub(super) type Username<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], T::AccountId>;

	// ACTION #7: Remaining storage items.

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(100)]
		pub fn new_player(origin: OriginFor<T>, username: [u8; 16]) -> DispatchResult{
			Self::is_username(username)?;
			let sender = ensure_signed(origin)?;
			Self::create_new_player(&sender, username)?;
			Ok(())
		}

	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {

		pub fn is_username(username: [u8; 16]) -> Result<bool, Error<T>> {
			match Self::username(username) {
				Some(_) => Err(<Error<T>>::UsernameUsed),
				None => Ok(true),
			}
		}

		pub fn create_new_player(sender: &T::AccountId, username: [u8; 16]) -> Result<(), Error<T>>{
			let player = Player::<T> {
				owner: sender.clone(),
				username,
				level: 0u32,
				exp: 0u32,
			};

			<Players<T>>::insert(sender, player);
			<Username<T>>::insert(username, sender);
			let new_cnt = Self::player_cnt().checked_add(1).ok_or(<Error<T>>::PlayerOverflow)?;
			<PlayerCnt<T>>::put(new_cnt);

			Self::deposit_event(Event::PlayerCreated(sender.clone(), username));
			Ok(())
		}
	}
}
