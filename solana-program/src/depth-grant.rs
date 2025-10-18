//! Depth Protocol Grant Engine (DGE) Program
//! This program implements a deterministic, trust-minimized DAO grant system
//! on Solana using the Builder Bond (BB) and the D-Metric for conditional payouts.

use anchor_lang::prelude::*;

// This is the program ID for the DGE.
declare_id!("DGE1111111111111111111111111111111111111111111111111111111111");

/// The primary DGE program module.
#[program]
pub mod depth_protocol_grant_engine {
    use super::*;

    /// Initializes a new grant account, staking the Builder Bond.
    /// The D-Metric philosophy ("Proof of Growth" or "verifiable conscious acts")
    /// is encoded here by defining the required initial criteria (e.g., code commits,
    /// governance vote participation, initial deposit).
    pub fn initialize_grant(
        ctx: Context<InitializeGrant>,
        total_grant_amount: u64,
        milestone_tranches: u8,
        builder_bond_amount: u64,
    ) -> Result<()> {
        let grant = &mut ctx.accounts.grant;

        // --- THE BUILDER BOND (BB) MECHANISM ---
        // This is the economic commitment from the builder, ensuring skin in the game.
        grant.builder = ctx.accounts.builder.key();
        grant.total_grant_amount = total_grant_amount;
        grant.builder_bond_amount = builder_bond_amount; // The amount staked for the BB
        grant.tranches_completed = 0;
        grant.total_tranches = milestone_tranches;
        grant.is_liquidated = false;

        // In a real-world scenario, a transfer instruction would move the builder_bond_amount
        // from the builder's account to the program's vault or the Grant account.
        msg!("Grant initialized. Builder Bond of {} staked successfully.", builder_bond_amount);
        Ok(())
    }

    /// Attempts to disburse the next grant tranche upon milestone completion.
    /// This function performs the core Cypherpunk accountability check via the D-Metric.
    /// Disbursal is **conditional** and **automated**.
    pub fn complete_milestone_and_payout(
        ctx: Context<MilestonePayout>,
        milestone_index: u8,
        on_chain_metric_data: u64, // Placeholder for verifiable data (e.g., unique user count)
    ) -> Result<()> {
        let grant = &mut ctx.accounts.grant;

        // Check if the grant is already marked for liquidation.
        if grant.is_liquidated {
            return err!(DGEError::GrantLiquidated);
        }

        // Ensure the correct sequence of milestones.
        if milestone_index != grant.tranches_completed + 1 {
            return err!(DGEError::MilestoneOutOfOrder);
        }

        // --- THE D-METRIC CHECK (Proof of Growth / Verifiable Conscious Acts) ---
        // This logic replaces subjective human review with a deterministic, auditable check.
        // The D-Metric minimum threshold is set here (e.g., a score of 75/100 or a specific metric value).
        let d_metric_score = calculate_d_metric(on_chain_metric_data);

        if d_metric_score < 75 {
            // D-Metric failed: Trigger Builder Bond liquidation and pause all future payouts.
            grant.is_liquidated = true;
            msg!("D-Metric failure (Score: {}). Builder Bond Liquidation Triggered. Grant Paused.", d_metric_score);

            // In a real program, an instruction would handle the liquidation of the bond,
            // returning it to the DAO treasury.
            return err!(DGEError::DMetricFailed);
        }

        // D-Metric passed: Proceed with tranche disbursement.
        grant.tranches_completed = milestone_index;
        let tranche_amount = grant.total_grant_amount / grant.total_tranches as u64;

        // In a real program, a CPI would transfer the tranche_amount from the
        // DAO's vault to the builder's account.
        msg!("D-Metric passed (Score: {}). Tranche {} of {} disbursed: {} SOL.",
            d_metric_score, grant.tranches_completed, grant.total_tranches, tranche_amount
        );

        Ok(())
    }

    /// Function to explicitly liquidate the Builder Bond if the D-Metric fails an off-chain audit (rare fallback).
    /// This is an emergency function and should be guarded by a secure DAO multisig.
    pub fn liquidate_bond(ctx: Context<LiquidateBond>) -> Result<()> {
        let grant = &mut ctx.accounts.grant;
        if grant.is_liquidated {
            msg!("Grant is already liquidated.");
            return Ok(());
        }

        // Mark the grant for liquidation and pause payouts.
        grant.is_liquidated = true;

        // Log the event for maximum transparency and auditability.
        msg!("Builder Bond Liquidation initiated by DAO Quorum.");

        // Real-world: Transfer the grant funds back to the DAO treasury.

        Ok(())
    }
}

// --- D-METRIC LOGIC IMPLEMENTATION ---

/// Calculates the deterministic D-Metric score based on verifiable on-chain data.
/// The core of the "verifiable conscious acts" philosophy is contained here.
/// This function must be purely deterministic and auditable.
fn calculate_d_metric(metric_data: u64) -> u8 {
    // Placeholder implementation:
    // This is where we would check things like:
    // 1. Has the associated BPF program been deployed?
    // 2. Has the project received X unique transactions?
    // 3. Has the team updated a verifiable on-chain registry?

    if metric_data >= 1000 {
        return 95; // Excellent verifiable progress
    } else if metric_data >= 500 {
        return 80; // Good verifiable progress
    } else {
        return 60; // Failure to meet "Proof of Growth" threshold
    }
}


// --- ACCOUNTS & DATA STRUCTURES ---

/// Context for initializing a new grant.
#[derive(Accounts)]
pub struct InitializeGrant<'info> {
    #[account(init, payer = builder, space = 8 + Grant::LEN)]
    pub grant: Account<'info, Grant>,
    /// CHECK: The builder is the one who pays for the account creation and stakes the bond.
    #[account(mut)]
    pub builder: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Context for milestone completion and payout.
#[derive(Accounts)]
pub struct MilestonePayout<'info> {
    #[account(mut, has_one = builder)]
    pub grant: Account<'info, Grant>,
    pub builder: Signer<'info>,
    /// CHECK: The DAO authority for treasury disbursement (used in a real CPI).
    #[account(mut)]
    pub dao_treasury: UncheckedAccount<'info>,
}

/// Context for bond liquidation (emergency fallback).
#[derive(Accounts)]
pub struct LiquidateBond<'info> {
    #[account(mut)]
    pub grant: Account<'info, Grant>,
    /// CHECK: Must be a signer from the DAO's multisig/governance program.
    pub dao_authority: Signer<'info>,
}

/// The main Grant Account data structure.
#[account]
pub struct Grant {
    pub builder: Pubkey,            // 32
    pub total_grant_amount: u64,    // 8
    pub builder_bond_amount: u64,   // 8
    pub tranches_completed: u8,     // 1
    pub total_tranches: u8,         // 1
    pub is_liquidated: bool,        // 1
    // Padding to ensure future expansion: ~40 bytes
}

impl Grant {
    pub const LEN: usize = 32 + 8 + 8 + 1 + 1 + 1 + 40;
}

// --- ERROR HANDLING ---

#[error_code]
pub enum DGEError {
    #[msg("The calculated D-Metric score is below the required threshold, triggering bond liquidation.")]
    DMetricFailed,
    #[msg("Cannot disburse a tranche; the grant has been liquidated and paused.")]
    GrantLiquidated,
    #[msg("Milestone submission is out of the required sequential order.")]
    MilestoneOutOfOrder,
}
