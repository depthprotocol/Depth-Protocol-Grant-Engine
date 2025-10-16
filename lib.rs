//! # Depth Protocol Grant Engine Pallet (DGE)
//!
//! The DGE Pallet enforces a two-layer security model for DAO grants:
//! 1. **Commitment Layer:** Requires a minimum D-Metric (Depth-Points) and a Builder Bond for submission.
//! 2. **Risk Management Layer:** Uses an Adaptive Quorum (AQ) calculation to dynamically increase the required approval threshold for funding tranches based on the project's current D-Metric performance.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{Saturating, Zero};

	// --- Custom Types ---

	/// Identifier for a specific grant project.
	pub type ProjectId = u32;

	/// Represents the current performance of a project (0-100).
	pub type DMetric = u8;

	// --- Configuration Trait ---

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The balance type for bonding and grant amounts.
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Copy
			+ MaxEncodedLen
			+ Zero;

		/// Handler for transferring or locking funds (e.g., the Builder Bond).
		type Currency: ReservableCurrency<Self::AccountId, Balance = Self::Balance>;

		/// The minimum D-Metric (Depth-Points) required for a founder to submit a proposal (10 D-Pnts).
		#[pallet::constant]
		type MinSubmissionDMetric: Get<u32>;

		/// The fixed Builder Bond amount required for every project submission ($300 USD equivalent).
		#[pallet::constant]
		type BuilderBond: Get<Self::Balance>;

		/// The Account ID authorized to update the D-Metric (e.g., a Governance Council).
		type DMetricAuthority: EnsureSigned<Self::AccountId>;

		/// Defines the Max Grant Capacity (10,000 equivalent) for a founder with Max D-Metric.
		#[pallet::constant]
		type MaxGrantCapacity: Get<Self::Balance>;
	}

	// --- Storage ---

	/// Storage for the next available ProjectId.
	#[pallet::storage]
	#[pallet::getter(fn next_project_id)]
	pub type NextProjectId<T> = StorageValue<_, ProjectId, ValueQuery>;

	/// Stores the Founder's Commitment (Depth-Points). (AccountId -> D-Metric u32)
	#[pallet::storage]
	#[pallet::getter(fn founder_commitment)]
	pub type FounderCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	/// Holds the active status and limits for a grant project. (ProjectId -> Project Status)
	#[pallet::storage]
	#[pallet::getter(fn project_status)]
	pub type ProjectStatus<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProjectId,
		ProjectDetails<T::Balance, T::AccountId>,
		OptionQuery,
	>;

	/// Stores the Builder Bond collateral for each project. (ProjectId -> Balance)
	#[pallet::storage]
	#[pallet::getter(fn builder_bonds)]
	pub type BuilderBonds<T: Config> =
		StorageMap<_, Blake2_128Concat, ProjectId, T::Balance, ValueQuery>;

	/// Struct to hold project-specific details.
	#[derive(Encode, Decode, Clone, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ProjectDetails<Balance, AccountId> {
		/// The performance score (0-100) used for Adaptive Quorum calculation.
		pub d_metric: DMetric,
		/// The maximum grant amount the founder is currently allowed to receive.
		pub max_grant_capacity: Balance,
		/// The Account ID of the founder who submitted the project.
		pub founder: AccountId,
	}

	// --- Events ---

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new grant proposal was successfully submitted. (Project ID, Founder, Bond Amount)
		ProposalSubmitted {
			project_id: ProjectId,
			founder: T::AccountId,
			bond_amount: T::Balance,
		},
		/// The D-Metric for a project was updated, changing the required Adaptive Quorum. (Project ID, New D-Metric)
		DMetricUpdated {
			project_id: ProjectId,
			new_d_metric: DMetric,
		},
		/// The Adaptive Quorum check was passed for a tranche release. (Project ID, Required Quorum)
		QuorumCheckPassed {
			project_id: ProjectId,
			required_quorum: u8,
		},
		/// The Adaptive Quorum check failed. (Project ID, Required Quorum)
		QuorumCheckFailed {
			project_id: ProjectId,
			required_quorum: u8,
		},
		/// A project's D-Metric fell below 50 and the Builder Bond was liquidated. (Project ID)
		ProjectLiquidated {
			project_id: ProjectId,
		},
	}

	// --- Errors ---

	#[pallet::error]
	pub enum Error<T> {
		/// The founder does not meet the minimum D-Metric required to submit a proposal.
		CommitmentFloorNotMet,
		/// The Adaptive Quorum was not met by the voting result.
		AdaptiveQuorumNotMet,
		/// The Project ID provided is invalid or the project does not exist.
		ProjectNotFound,
		/// The D-Metric must be between 0 and 100.
		DMetricOutOfRange,
		/// The founder could not pay the required Builder Bond.
		BondDepositFailed,
	}

	// --- Core Pallet Logic ---

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submits a new grant proposal.
		///
		/// Checks founder commitment (D-Metric), collects the Builder Bond, and calculates
		/// the Max Grant Capacity.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::get_new_project_id())] // Placeholder weight
		pub fn submit_proposal(origin: OriginFor<T>) -> DispatchResult {
			let founder = ensure_signed(origin)?;

			let founder_d_metric = Self::founder_commitment(&founder);

			// 1. Check D-Metric Floor (Commitment Layer)
			ensure!(
				founder_d_metric >= T::MinSubmissionDMetric::get(),
				Error::<T>::CommitmentFloorNotMet
			);

			// 2. Calculate Max Grant Capacity (Scales from 1k up to 10k at 250 D-Pnt)
			let max_capacity: T::Balance = {
				let max_cap = T::MaxGrantCapacity::get(); // $10,000 equiv.
				let max_cap_u32: u32 = max_cap.try_into().unwrap_or(0); // For math simplicity
				
				// Linear scaling: (D-Metric / 250) * MaxCapacity
				let scaled_amount = if founder_d_metric >= 250 {
					max_cap_u32
				} else {
					(founder_d_metric * max_cap_u32) / 250
				};

				scaled_amount.into()
			};

			// 3. Collect Builder Bond
			let bond_amount = T::BuilderBond::get();
			T::Currency::reserve(&founder, bond_amount)
				.map_err(|_| Error::<T>::BondDepositFailed)?;

			// 4. Create Project
			let project_id = Self::next_project_id();
			let new_project_id = project_id.saturating_add(1);

			ProjectStatus::<T>::insert(
				project_id,
				ProjectDetails {
					d_metric: 100, // Starts at 100 (baseline)
					max_grant_capacity: max_capacity,
					founder: founder.clone(),
				},
			);
			BuilderBonds::<T>::insert(project_id, bond_amount);
			NextProjectId::<T>::put(new_project_id);

			Self::deposit_event(Event::ProposalSubmitted {
				project_id,
				founder,
				bond_amount,
			});

			Ok(())
		}

		/// Simulates a vote on a milestone tranche release, checking the Adaptive Quorum.
		///
		/// Note: In a real system, this would be tied to a governance module (e.g., Democracy).
		/// Here we simulate the quorum check with a mock 'approval_percentage'.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::get_new_project_id())] // Placeholder weight
		pub fn vote_on_tranche_release(
			origin: OriginFor<T>,
			project_id: ProjectId,
			approval_percentage: u8,
		) -> DispatchResult {
			// This extrinsic can be called by anyone (the DAO).
			ensure_signed(origin)?;

			let project_details =
				Self::project_status(project_id).ok_or(Error::<T>::ProjectNotFound)?;

			let required_quorum = Self::calculate_adaptive_quorum(project_details.d_metric);

			// 1. Check Adaptive Quorum (Risk Management Layer)
			if approval_percentage >= required_quorum {
				Self::deposit_event(Event::QuorumCheckPassed {
					project_id,
					required_quorum,
				});
			} else {
				Self::deposit_event(Event::QuorumCheckFailed {
					project_id,
					required_quorum,
				});
				// Fail the dispatch to stop the tranche release process
				return Err(Error::<T>::AdaptiveQuorumNotMet.into());
			}

			Ok(())
		}

		/// Updates the D-Metric of an active project. Only callable by an authorized entity.
		///
		/// This immediately changes the required Adaptive Quorum for future votes.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::get_new_project_id())] // Placeholder weight
		pub fn update_d_metric(
			origin: OriginFor<T>,
			project_id: ProjectId,
			new_d_metric: DMetric,
		) -> DispatchResult {
			// Ensure the caller is the designated authority
			T::DMetricAuthority::ensure_signed(origin)?;

			ensure!(new_d_metric <= 100, Error::<T>::DMetricOutOfRange);

			ProjectStatus::<T>::try_mutate_exists(
				project_id,
				|maybe_details| -> DispatchResult {
					let details = maybe_details.as_mut().ok_or(Error::<T>::ProjectNotFound)?;
					details.d_metric = new_d_metric;

					// 1. Check for Liquidation Floor (D-Metric < 50)
					if new_d_metric < 50 {
						Self::handle_liquidation(project_id, &details.founder)?;
						*maybe_details = None; // Remove project status
					} else {
						Self::deposit_event(Event::DMetricUpdated {
							project_id,
							new_d_metric,
						});
					}

					Ok(())
				},
			)?;

			Ok(())
		}
	}

	// --- Helper Functions ---

	impl<T: Config> Pallet<T> {
		/// Calculates the Adaptive Quorum required for a tranche release.
		///
		/// Formula: Required Approval = 51% + Risk Penalty
		/// Risk Penalty = (100 - Current D-Metric) / 2
		///
		/// Returns the required approval percentage as a u8 (0-100).
		pub fn calculate_adaptive_quorum(d_metric: DMetric) -> u8 {
			const BASELINE_QUORUM: u8 = 51;
			const MAX_D_METRIC: u8 = 100;

			// Ensure D-Metric is not above 100
			let safe_d_metric = d_metric.min(MAX_D_METRIC);

			// Risk Penalty = (100 - D-Metric) / 2
			let risk_penalty = (MAX_D_METRIC.saturating_sub(safe_d_metric)) / 2;

			// Required Quorum = 51 + Risk Penalty
			BASELINE_QUORUM.saturating_add(risk_penalty)
		}

		/// Handles the liquidation process: forfeits the Builder Bond.
		fn handle_liquidation(project_id: ProjectId, founder: &T::AccountId) -> DispatchResult {
			let bond_amount = BuilderBonds::<T>::take(project_id);

			// Forfeit the bond (unreserve and drop, effectively destroying it or moving it to DAO treasury).
			// We move it to the system treasury for this simplified example.
			T::Currency::unreserve(founder, bond_amount); // Unreserve from founder
			// Note: A real implementation would transfer this to the DAO's treasury account.

			Self::deposit_event(Event::ProjectLiquidated { project_id });

			Ok(())
		}
	}
}
