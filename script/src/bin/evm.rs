//! Generate an EVM-compatible proof (Groth16 or PLONK) for on-chain verification.
//!
//! Usage:
//!   cargo run --release --bin evm -- --system groth16
//!   cargo run --release --bin evm -- --system plonk

use alloy_sol_types::SolType;
use clap::{Parser, ValueEnum};
use fibonacci_lib::SimulationProof;
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, HashableKey, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;

const SIMULATION_ELF: Elf = include_elf!("fibonacci-program");

#[derive(Parser, Debug)]
#[command(author, version, about = "Generate EVM-compatible antelligence ZK proof")]
struct EVMArgs {
    #[arg(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1SimulationProofFixture {
    kill_rate: u32,
    nanobot_count: u16,
    tumor_radius: u16,
    steps: u32,
    valid: bool,
    vkey: String,
    public_values: String,
    proof: String,
}

fn main() {
    sp1_sdk::utils::setup_logger();

    let args = EVMArgs::parse();
    let client = ProverClient::from_env();
    let pk = client.setup(SIMULATION_ELF).expect("failed to setup elf");

    // Test simulation inputs
    let config_bytes: Vec<u8> = b"test-config-tumor-150um-10bots".to_vec();
    let tumor_radius: u32 = 150;
    let nanobot_count: u32 = 10;
    let steps: u32 = 300;
    let seed: u64 = 42;
    let oxygen_level_x1000: u32 = 38_000;
    let drug_dosage_x1000: u32 = 90_000;
    let kills: u32 = 30;
    let total_cells: u32 = 66;
    let kill_rate: u32 = (kills as u64 * 10000 / total_cells as u64) as u32;

    let mut stdin = SP1Stdin::new();
    stdin.write(&config_bytes);
    stdin.write(&tumor_radius);
    stdin.write(&nanobot_count);
    stdin.write(&steps);
    stdin.write(&seed);
    stdin.write(&oxygen_level_x1000);
    stdin.write(&drug_dosage_x1000);
    stdin.write(&kills);
    stdin.write(&total_cells);
    stdin.write(&kill_rate);

    println!("Proof System: {:?}", args.system);

    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, pk.verifying_key(), args.system);
}

fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
) {
    let bytes = proof.public_values.as_slice();
    let decoded = SimulationProof::abi_decode(bytes).unwrap();

    let fixture = SP1SimulationProofFixture {
        kill_rate: decoded.killRate,
        nanobot_count: decoded.nanobotCount,
        tumor_radius: decoded.tumorRadius,
        steps: decoded.steps,
        valid: decoded.valid,
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    println!("Verification Key: {}", fixture.vkey);
    println!("Public Values: {}", fixture.public_values);
    println!("Proof Bytes: {}", fixture.proof);

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
