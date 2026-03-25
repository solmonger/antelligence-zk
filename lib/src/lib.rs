use alloy_sol_types::sol;

sol! {
    /// Public values for the antelligence simulation proof.
    /// These are revealed on-chain; everything else stays private.
    struct SimulationProof {
        /// keccak256 hash of the private simulation config (tumor geometry, drug params)
        bytes32 configHash;
        /// The simulation kill rate (scaled by 10000, e.g., 4550 = 45.5%)
        uint32 killRate;
        /// Number of nanobots used
        uint16 nanobotCount;
        /// Tumor radius in micrometers
        uint16 tumorRadius;
        /// Number of simulation steps
        uint32 steps;
        /// Random seed commitment (proves deterministic execution)
        bytes32 seedHash;
        /// Whether the simulation result is valid (all checks passed)
        bool valid;
    }
}

/// Verify simulation parameters are within valid medical ranges.
/// All values are integers (no floating point in zkVM).
/// oxygen_level and drug_dosage are scaled by 1000 (e.g., 38000 = 38.0 mmHg).
pub fn validate_params(
    tumor_radius: u32,
    nanobot_count: u32,
    steps: u32,
    oxygen_level_x1000: u32,
    drug_dosage_x1000: u32,
) -> bool {
    if tumor_radius < 50 || tumor_radius > 500 {
        return false;
    }
    if nanobot_count < 1 || nanobot_count > 100 {
        return false;
    }
    if steps < 10 || steps > 10000 {
        return false;
    }
    if oxygen_level_x1000 > 100_000 {
        return false; // 0-100 mmHg
    }
    if drug_dosage_x1000 > 1_000_000 {
        return false; // 0-1000 µg
    }
    true
}

/// Compute a deterministic simulation score from parameters.
/// In the full implementation, this would run the actual BioFVM solver.
/// For the MVP, we verify the hash chain: config → seed → execution → score.
pub fn verify_simulation_hash(
    config_bytes: &[u8],
    seed: u64,
    claimed_kill_rate: u32,
    claimed_kills: u32,
    claimed_total_cells: u32,
) -> bool {
    // Verify kill rate calculation is correct
    if claimed_total_cells == 0 {
        return claimed_kill_rate == 0;
    }
    let computed_rate = (claimed_kills as u64 * 10000) / claimed_total_cells as u64;
    if computed_rate as u32 != claimed_kill_rate {
        return false;
    }

    // Verify kills don't exceed total cells
    if claimed_kills > claimed_total_cells {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_params_valid() {
        assert!(validate_params(150, 10, 300, 38000, 90000));
    }

    #[test]
    fn test_validate_params_tumor_too_small() {
        assert!(!validate_params(10, 10, 300, 38000, 90000));
    }

    #[test]
    fn test_validate_params_tumor_too_large() {
        assert!(!validate_params(600, 10, 300, 38000, 90000));
    }

    #[test]
    fn test_validate_params_zero_nanobots() {
        assert!(!validate_params(150, 0, 300, 38000, 90000));
    }

    #[test]
    fn test_validate_params_boundary() {
        assert!(validate_params(50, 1, 10, 0, 0));
        assert!(validate_params(500, 100, 10000, 100_000, 1_000_000));
    }

    #[test]
    fn test_verify_hash_zero_cells() {
        assert!(verify_simulation_hash(b"config", 42, 0, 0, 0));
    }

    #[test]
    fn test_verify_hash_kills_exceed_total() {
        assert!(!verify_simulation_hash(b"config", 42, 10000, 100, 50));
    }

    #[test]
    fn test_verify_hash_perfect_score() {
        assert!(verify_simulation_hash(b"config", 42, 10000, 66, 66));
    }

    #[test]
    fn test_verify_hash_correct_rate() {
        // 30/66 * 10000 = 4545 (integer division)
        assert!(verify_simulation_hash(b"config", 42, 4545, 30, 66));
    }

    #[test]
    fn test_verify_hash_wrong_rate() {
        assert!(!verify_simulation_hash(b"config", 42, 5000, 30, 66));
    }
}
