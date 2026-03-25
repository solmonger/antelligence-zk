//! Antelligence ZK Proof Program
//!
//! Proves that a BioFVM simulation was run correctly without revealing
//! private patient data (tumor geometry, drug parameters).
//!
//! Public outputs: configHash, killRate, nanobotCount, tumorRadius, steps, seedHash, valid
//! Private inputs: full config, seed, simulation results

#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use fibonacci_lib::{validate_params, verify_simulation_hash, SimulationProof};

/// Simple deterministic hash for binding commitments inside the zkVM.
/// Uses iterative XOR-shift mixing to produce a 32-byte digest.
fn simple_hash(data: &[u8]) -> [u8; 32] {
    let mut h = [0u8; 32];
    for (i, &b) in data.iter().enumerate() {
        let idx = i % 32;
        h[idx] = h[idx].wrapping_add(b).wrapping_mul(31);
        h[(idx + 7) % 32] ^= b.wrapping_add(i as u8);
    }
    h
}

pub fn main() {
    // ========================================
    // READ PRIVATE INPUTS (not revealed)
    // ========================================

    let config_bytes: Vec<u8> = sp1_zkvm::io::read();
    let tumor_radius: u32 = sp1_zkvm::io::read();
    let nanobot_count: u32 = sp1_zkvm::io::read();
    let steps: u32 = sp1_zkvm::io::read();
    let seed: u64 = sp1_zkvm::io::read();
    let oxygen_level_x1000: u32 = sp1_zkvm::io::read();
    let drug_dosage_x1000: u32 = sp1_zkvm::io::read();

    // Private: simulation results
    let kills: u32 = sp1_zkvm::io::read();
    let total_cells: u32 = sp1_zkvm::io::read();
    let kill_rate: u32 = sp1_zkvm::io::read(); // scaled by 10000

    // ========================================
    // VERIFY (inside the zkVM)
    // ========================================

    let params_valid = validate_params(
        tumor_radius, nanobot_count, steps, oxygen_level_x1000, drug_dosage_x1000,
    );

    let results_valid = verify_simulation_hash(
        &config_bytes, seed, kill_rate, kills, total_cells,
    );

    let valid = params_valid && results_valid;

    // Compute commitment hashes via simple deterministic hash
    // (The hash binds the proof to specific inputs without revealing them)
    let config_hash = simple_hash(&config_bytes);
    let seed_hash = simple_hash(&seed.to_le_bytes());

    // ========================================
    // COMMIT PUBLIC VALUES (revealed on-chain)
    // ========================================

    let proof = SimulationProof {
        configHash: config_hash.into(),
        killRate: kill_rate,
        nanobotCount: nanobot_count as u16,
        tumorRadius: tumor_radius as u16,
        steps,
        seedHash: seed_hash.into(),
        valid,
    };

    sp1_zkvm::io::commit_slice(&SimulationProof::abi_encode(&proof));
}
