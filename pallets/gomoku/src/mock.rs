use crate as pallet_gomoku;
use frame_support::parameter_types;
use frame_system as system;

use frame_support::traits::{Currency, OnFinalize, OnInitialize};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

pub use pallet_balances::Call as BalancesCall;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		PalletGame: pallet_gomoku::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		// Event: Event,
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type AccountData = pallet_balances::AccountData<u64>;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const MaxGomokuPlayer: u32 = 2;
	pub const MaxGame: u32 = 10;
	pub const MaxOpenGame: u32 = 10;
	pub const MaxStartGame: u32 = 10;
	pub const OpenGameFee: u32 = 1000000000u32;
	pub const MaxEndedGame: u32 = 1000000000u32;
}

impl pallet_gomoku::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MaxGomokuPlayer = MaxGomokuPlayer;
	type MaxGame = MaxGame;
	type MaxOpenGame = MaxOpenGame;
	type MaxStartGame = MaxStartGame;
	type OpenGameFee = OpenGameFee;
	type MaxEndedGame = MaxEndedGame;
	type GameRandomness = RandomnessCollectiveFlip;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			PalletGame::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		PalletGame::on_initialize(System::block_number());
	}
}

pub struct ExtBuilder;

// impl ExtBuilder {
// 	pub fn build(self) -> sp_io::TestExternalities {
// 		let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 		let mut ext = sp_io::TestExternalities::new(t);
// 		ext.execute_with(|| System::set_block_number(1));
// 		ext
// 	}
// }

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(ALICE, 1000000000), (BOB, 1000000000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();


		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext

	}
}
