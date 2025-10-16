//! The Substrate Node Template runtime.
//! This file configures and defines the Depth Grant Engine (DGE) runtime.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does not need `()` in the module list
#![allow(clippy::no_mangle_with_rust_abi)]
#![allow(clippy::too_many_lines)]

// Make the DGE pallet available to the runtime.
use pallet_dge;

use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, IdentifyAccount, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few more imports needed for the DGE Pallet Configuration
use frame_support::traits::{Currency, Imbalance, Nothing, OnUnbalanced, StorageInfo};
use frame_support::weights::{
	constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
	IdentityFee, Weight,
};
use frame_support::{construct_runtime, parameter_types};

// --- Standard Types ---

/// Alias to the Block number
pub type BlockNumber = u32;

/// Alias to the signature type in the runtime.
pub type Signature = sp_runtime::MultiSignature;

/// Alias to the account identifier type.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The address format for all runtime accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

/// Balance of an account.
pub type Balance = u128; // Using u128 for high-precision currency

/// Index of a transaction in the chain.
pub type Index = u32;

/// The hash of a block's header.
pub type Hash = sp_core::H256;

// --- DGE Configuration Constants ---

/// The canonical unit of the DPT token (e.g., 10^12).
pub const TOKEN_UNIT: Balance = 1_000_000_000_000;

parameter_types! {
	// The fixed collateral required to submit any grant proposal (approx $300 USD equiv).
	pub const BuilderBond: Balance = 300 * TOKEN_UNIT;

	// The minimum D-Metric (Depth-Points) a founder must have to submit a proposal.
	pub const MinDMetric: u32 = 10;

	// The maximum grant size a 250 D-Metric founder can request ($10,000 USD equiv).
	pub const MaxGrantCapacity: Balance = 10_000 * TOKEN_UNIT;
}

// --- Frame System Configuration ---

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = RuntimeVersion {
		spec_version: 100,
		impl_version: 100,
		authoring_version: 1,
		..
	};
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = RocksDbWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// --- Balances Pallet Configuration (Currency Dependency) ---

parameter_types! {
	pub const ExistentialDeposit: Balance = 500 * TOKEN_UNIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	/// The type for the Balance pallet's events.
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = Nothing;
	type MaxFreezes = ConstU32<0>;
	type MaxHolds = ConstU32<0>;
	type RuntimeHoldReason = Nothing;
}

// --- DGE Pallet Configuration (Our Custom Logic) ---

impl pallet_dge::Config for Runtime {
	/// The type for the DGE pallet's events.
	type RuntimeEvent = RuntimeEvent;

	/// Uses the Balance type defined globally for the runtime.
	type Balance = Balance;

	/// Uses the Balances pallet as the currency handler for the Builder Bond reservation.
	type Currency = Balances;

	/// The constant for the minimum D-Metric (10).
	type MinSubmissionDMetric = MinDMetric;

	/// The constant for the required Builder Bond (300 DPT equivalent).
	type BuilderBond = BuilderBond;

	/// Defines the authorized entity for updating the D-Metric.
	/// We use `EnsureRoot` (Sudo) for testing and simplicity, but this should be
	/// replaced by a dedicated DAO council or multisig in production.
	type DMetricAuthority = frame_system::EnsureRoot<AccountId>;

	/// The constant for the maximum grant capacity (10,000 DPT equivalent).
	type MaxGrantCapacity = MaxGrantCapacity;
}

// --- Other Pallets (Needed for a Functional Node) ---

impl pallet_timestamp::Config for Runtime {
	/// The type for the pallet's events.
	type RuntimeEvent = RuntimeEvent;
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<{ 5_000 }>; // 5 second minimum period
	type WeightInfo = ();
}

// --- Construct Runtime Macro ---

// The critical step: bringing all the configured pallets together.
construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,

		// The Depth Grant Engine pallet (our main logic)
		Dge: pallet_dge,
	}
);

// --- Standard Substrate Node Boilerplate (omitted for brevity, but necessary for a full node) ---

// ... (Rest of the standard runtime code, including transaction validity,
// opaque types, and runtime API implementations, would go here).
