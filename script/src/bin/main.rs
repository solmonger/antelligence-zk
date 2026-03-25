//! Antelligence ZK Script — Execute or prove the simulation verification program.
//!
//! Usage:
//!   cargo run --release -- --execute    # Execute without proof
//!   cargo run --release -- --prove      # Generate SP1 core proof

use alloy_sol_types::SolType;
use clap::Parser;
use fibonacci_lib::SimulationProof;
use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, ProvingKey, SP1Stdin,
};

/// The ELF for the antelligence simulation proof program.
const SIMULATION_ELF: Elf = include_elf!("fibonacci-program");

#[derive(Parser, Debug)]
#[command(author, version, about = "Antelligence ZK proof script")]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();
    if args.execute == args.prove {
        eprintln!("Error: specify either --execute or --prove");
        std::process::exit(1);
    }

    let client = ProverClient::from_env();

    // Setup test simulation inputs
    let config_bytes: Vec<u8> = b"test-config-tumor-150um-10bots".to_vec();
    let tumor_radius: u32 = 150;
    let nanobot_count: u32 = 10;
    let steps: u32 = 300;
    let seed: u64 = 42;
    let oxygen_level_x1000: u32 = 38_000; // 38.0 mmHg
    let drug_dosage_x1000: u32 = 90_000;  // 90.0 ug

    // Simulation results (would come from BioFVM in production)
    let kills: u32 = 30;
    let total_cells: u32 = 66;
    let kill_rate: u32 = (kills as u64 * 10000 / total_cells as u64) as u32; // 4545

    // Write inputs to SP1 stdin
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

    println!("Antelligence ZK Simulation Proof");
    println!("  Tumor radius: {}um", tumor_radius);
    println!("  Nanobots: {}", nanobot_count);
    println!("  Steps: {}", steps);
    println!("  Kill rate: {:.1}%", kill_rate as f64 / 100.0);

    if args.execute {
        let (output, report) = client.execute(SIMULATION_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        let decoded = SimulationProof::abi_decode(output.as_slice()).unwrap();
        println!("Public values:");
        println!("  Kill rate: {:.1}%", decoded.killRate as f64 / 100.0);
        println!("  Nanobot count: {}", decoded.nanobotCount);
        println!("  Tumor radius: {}um", decoded.tumorRadius);
        println!("  Steps: {}", decoded.steps);
        println!("  Valid: {}", decoded.valid);
        println!("Cycles: {}", report.total_instruction_count());

        assert!(decoded.valid, "Simulation validation failed!");
        assert_eq!(decoded.killRate, kill_rate);
        println!("All assertions passed!");
    } else {
        let pk = client.setup(SIMULATION_ELF).expect("failed to setup elf");
        let proof = client.prove(&pk, stdin).run().expect("failed to generate proof");
        println!("Successfully generated proof!");

        client.verify(&proof, pk.verifying_key(), None).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
