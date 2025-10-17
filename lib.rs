// Proof-of-Depth Governance Module
// This module provides a single, core function: calculate_adaptive_quorum.
// It dynamically adjusts the required governance quorum percentage based on
// the protocol's total Depth Score (a measure of TVL/staking).

// We use the 'decimal-fixed-point' crate for precise, on-chain calculations
// without using floating-point numbers, which are typically prohibited in smart contracts.
// For this example, we will simulate the fixed-point math using standard integers,
// where the result is scaled by 10^18 (the common fixed-point scale).

// --- Fixed-Point Constants ---

// Scale factor for fixed-point math. A value of 1_000_000_000_000_000_000 represents 1.0.
const SCALE: u128 = 1_000_000_000_000_000_000;

// --- Adaptive Quorum (AQ) Constants ---

// The lowest possible Quorum percentage (Floor), represented as a scaled value.
const QUORUM_FLOOR: u128 = 150_000_000_000_000_000; // 0.15 * SCALE (15%)

// The highest possible Quorum percentage (Ceiling), represented as a scaled value.
const QUORUM_CEILING: u128 = 450_000_000_000_000_000; // 0.45 * SCALE (45%)

// The depth threshold (K) at which the quorum curve begins to flatten significantly.
const DEPTH_THRESHOLD_K: u64 = 100_000_000; // $100 Million USD (Example threshold)

// The difference between the ceiling and the floor
const CEILING_MINUS_FLOOR: u128 = QUORUM_CEILING - QUORUM_FLOOR;

// --- Builder Bond (BB) Constants ---

// The required Builder Bond amount for a Tier 1 (Community) App, in USD equivalent.
const TIER_1_BUILDER_BOND_USD: u64 = 300; 

// --- Core Public Functions ---

/// Calculates the required Adaptive Quorum percentage based on the protocol's Depth Score.
/// The logic implements a simplified decay curve to scale the quorum between 15% and 45%.
/// [The full calculation logic remains the same as previously defined]
pub fn calculate_adaptive_quorum(total_protocol_depth: u64) -> u128 {
    // 1. Calculate the normalized Depth Score (x = Depth / K).
    let depth_u128: u128 = total_protocol_depth as u128;
    let threshold_u128: u128 = DEPTH_THRESHOLD_K as u128;

    let ratio_x: u128;
    if total_protocol_depth == 0 {
        ratio_x = 0;
    } else {
        ratio_x = (depth_u128.checked_mul(SCALE).unwrap_or(u128::MAX)) / threshold_u128;
    }

    // A simplified, inverse power decay function is used to approximate the curve: f(x) = SCALE / (SCALE + x)
    let decay_term: u128;

    let denominator_check = SCALE.checked_add(ratio_x);
    if denominator_check.is_none() {
        decay_term = 0;
    } else {
        let denominator = denominator_check.unwrap();
        decay_term = (SCALE.checked_mul(SCALE).unwrap_or(u128::MAX)) / denominator;
    }

    // 2. Apply the decay term to the Quorum range:
    let weighted_quorum = (decay_term.checked_mul(CEILING_MINUS_FLOOR).unwrap_or(u128::MAX)) / SCALE;

    // 3. Add the Floor to get the final Adaptive Quorum:
    let final_quorum = QUORUM_FLOOR.checked_add(weighted_quorum).unwrap_or(u128::MAX);

    final_quorum.min(QUORUM_CEILING)
}

/// Calculates the amount of $FST tokens required for the Builder Bond.
/// 
/// This function assumes the protocol can retrieve the current market price of $FST in USD.
/// This would typically require an Oracle feed (e.g., Pyth Network, Chainlink) in a live deployment.
/// 
/// # Arguments
/// * `fst_price_in_usd_scaled` - The current price of 1 $FST token, scaled by 10^18.
///   (e.g., if FST is $1.50, this value is 1_500_000_000_000_000_000)
/// 
/// # Returns
/// * The amount of $FST tokens (as a raw token amount) required for the bond.
///   (This value must be divided by the FST token's decimal value on the frontend for display)
pub fn get_builder_bond_amount(fst_price_in_usd_scaled: u128) -> Result<u64, ProgramError> {
    
    // Bond Amount (FST) = (Bond Value USD * SCALE) / FST Price USD (Scaled)
    
    if fst_price_in_usd_scaled == 0 {
        // Prevent division by zero if the token price is somehow zero
        return Err(ProgramError::Custom(100)); // Price Oracle Error
    }

    let bond_usd_scaled = (TIER_1_BUILDER_BOND_USD as u128).checked_mul(SCALE).ok_or(ProgramError::Custom(101))?;

    // Perform the scaled division: (Bond_USD_Scaled * SCALE) / FST_Price_Scaled
    // We multiply by SCALE again to maintain precision through the division, then divide by SCALE at the end.
    let fst_amount_scaled = bond_usd_scaled.checked_mul(SCALE).ok_or(ProgramError::Custom(102))?
        .checked_div(fst_price_in_usd_scaled)
        .ok_or(ProgramError::Custom(103))?;
    
    // Un-scale the final FST token amount and convert to u64 for return
    let fst_amount_raw = (fst_amount_scaled.checked_div(SCALE).ok_or(ProgramError::Custom(104))?)
        .try_into()
        .map_err(|_| ProgramError::Custom(105))?; // Conversion Error
        
    Ok(fst_amount_raw)
}

// --- Testing Section (For review and verification) ---

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::program_error::ProgramError;

    // Helper functions (omitted for brevity, assume they are the same as before)
    fn quorum_to_percent_string(quorum: u128) -> String {
        let percent = quorum as f64 / SCALE as f64 * 100.0;
        format!("{:.2}%", percent)
    }
    
    // --- AQ Tests (omitted for brevity, assume same as before) ---
    // ... test_zero_depth, test_low_depth, etc.

    // --- Builder Bond Tests ---
    
    // Helper function to scale a price (e.g., $1.50 -> 1_500_000_000_000_000_000)
    fn scale_price(price: f64) -> u128 {
        (price * SCALE as f64).round() as u128
    }

    #[test]
    // Test case: FST token price is $1.00. Bond should be 300 FST.
    fn test_bond_price_one_dollar() -> Result<(), ProgramError> {
        let price = scale_price(1.0); // $1.00 USD
        let bond_amount = get_builder_bond_amount(price)?;
        assert_eq!(bond_amount, 300);
        println!("FST Price: $1.00 | Bond: {} FST", bond_amount);
        Ok(())
    }

    #[test]
    // Test case: FST token price is $1.50. Bond should be 200 FST.
    fn test_bond_price_one_fifty() -> Result<(), ProgramError> {
        let price = scale_price(1.5); // $1.50 USD
        let bond_amount = get_builder_bond_amount(price)?;
        assert_eq!(bond_amount, 200);
        println!("FST Price: $1.50 | Bond: {} FST", bond_amount);
        Ok(())
    }

    #[test]
    // Test case: FST token price is $0.50. Bond should be 600 FST.
    fn test_bond_price_fifty_cents() -> Result<(), ProgramError> {
        let price = scale_price(0.5); // $0.50 USD
        let bond_amount = get_builder_bond_amount(price)?;
        assert_eq!(bond_amount, 600);
        println!("FST Price: $0.50 | Bond: {} FST", bond_amount);
        Ok(())
    }
}
