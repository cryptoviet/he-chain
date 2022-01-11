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
	use sp_io::hashing::blake2_256;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// ACTION #1: Write a Struct to hold Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Hero<T: Config> {
		id: [u8; 32],
		owner: AccountOf<T>,
		name: [u8; 16],
		species: Species,
		class: Class,
		tier: Tier,
		level: u16,
		exp: u32,
	}

	// ACTION #2: Enum declaration for Gender.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Tier {
		Common,
		Rare,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Species {
		Dragon,
		Dwarf,
		Demon,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Class {
		Assassin,
		DemonHunter,
		Druid,
	}

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
		#[pallet::constant]
		type MaxHeroOwned: Get<u32>;
		type HeroRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		HeroIdExisted,
		ExceedMaxHeroOwned,
		HeroOverflow,
	}

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewHeroMinted(T::AccountId, [u8; 16], [u8; 32]),
	}

	#[pallet::storage]
	#[pallet::getter(fn hero_cnt)]
	pub(super) type HeroCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	// ACTION #7: Remaining storage items.
	#[pallet::storage]
	#[pallet::getter(fn heroes)]
	pub(super) type Heroes<T: Config> = StorageMap<_, Twox64Concat, [u8; 32], Hero<T>>;

	#[pallet::storage]
	#[pallet::getter(fn hero_owned)]
	pub(super) type HeroesOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<[u8; 32], T::MaxHeroOwned>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(100)]
		pub fn mint(origin: OriginFor<T>, name: [u8; 16]) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			let id = Self::mint_hero(sender.clone(), name)?;
			Self::deposit_event(Event::NewHeroMinted(sender, name, id));
			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {

		#[transactional]
		pub fn mint_hero(sender: T::AccountId, name: [u8; 16]) -> Result<[u8; 32], Error<T>> {
			let id = Self::gen_id()?;
			Self::is_id_available(id)?;

			<HeroesOwned<T>>::try_mutate(&sender, |hero_vec| {
				hero_vec.try_push(id)
			}).map_err(|_| <Error<T>>::ExceedMaxHeroOwned)?;

			let hero = Hero::<T> {
				id,
				owner: sender.clone(),
				name,
				species: Species::Demon,
				class: Class::Assassin,
				tier: Tier::Common,
				level: 0u16,
				exp: 0u32,
			};

			<Heroes<T>>::insert(id, hero);
			let new_hero_cnt = Self::hero_cnt().checked_add(1).ok_or(<Error<T>>::HeroOverflow)?;
			<HeroCnt<T>>::put(new_hero_cnt);

			Ok(id)
		}

		pub fn gen_id() -> Result<[u8; 32], Error<T>>{
			let payload = (
				T::HeroRandomness::random(&b""[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			Ok(payload.using_encoded(blake2_256))
		}

		pub fn is_id_available(id: [u8;32]) -> Result<bool, Error<T>> {
			match Self::heroes(id) {
				Some(_) => Err(<Error<T>>::HeroIdExisted),
				None => Ok(true)
			}
		}

	}
}
