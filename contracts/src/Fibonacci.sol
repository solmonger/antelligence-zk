// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

/// @notice Public values revealed by the ZK proof. Matches the Rust SimulationProof struct.
struct SimulationProof {
    bytes32 configHash;
    uint32 killRate;       // scaled by 10000 (e.g., 4550 = 45.5%)
    uint16 nanobotCount;
    uint16 tumorRadius;    // micrometers
    uint32 steps;
    bytes32 seedHash;
    bool valid;
}

/// @title TumorIntel — Privacy-Preserving Simulation Attestation
/// @author Antelligence (solmonger)
/// @notice Verifies ZK proofs that a BioFVM tumor simulation was run correctly,
///         without revealing private patient data (tumor geometry, drug parameters).
/// @dev Uses SP1 verifier gateway for proof verification on Base L2.
contract TumorIntel {
    /// @notice SP1 verifier contract address (SP1VerifierGateway on Base).
    address public verifier;

    /// @notice Verification key for the antelligence simulation program.
    bytes32 public simulationVKey;

    /// @notice Emitted when a simulation proof is verified on-chain.
    event SimulationVerified(
        bytes32 indexed configHash,
        uint32 killRate,
        uint16 nanobotCount,
        uint16 tumorRadius,
        uint32 steps,
        bool valid,
        address submitter
    );

    /// @notice Registry of verified simulation hashes.
    mapping(bytes32 => bool) public verifiedConfigs;

    constructor(address _verifier, bytes32 _simulationVKey) {
        verifier = _verifier;
        simulationVKey = _simulationVKey;
    }

    /// @notice Verify a simulation proof and register the result on-chain.
    /// @param _publicValues ABI-encoded SimulationProof struct.
    /// @param _proofBytes The SP1 proof bytes.
    /// @return proof The decoded simulation proof values.
    function verifySimulation(bytes calldata _publicValues, bytes calldata _proofBytes)
        public
        returns (SimulationProof memory proof)
    {
        // Verify the ZK proof against the SP1 verifier
        ISP1Verifier(verifier).verifyProof(simulationVKey, _publicValues, _proofBytes);

        // Decode public values
        proof = abi.decode(_publicValues, (SimulationProof));

        // Only register valid simulations
        require(proof.valid, "Simulation validation failed");

        // Register the config hash as verified
        verifiedConfigs[proof.configHash] = true;

        // Emit event for indexing
        emit SimulationVerified(
            proof.configHash,
            proof.killRate,
            proof.nanobotCount,
            proof.tumorRadius,
            proof.steps,
            proof.valid,
            msg.sender
        );
    }

    /// @notice Check if a simulation config has been verified.
    /// @param configHash The keccak256 hash of the simulation config.
    /// @return True if the config has a verified proof on-chain.
    function isVerified(bytes32 configHash) public view returns (bool) {
        return verifiedConfigs[configHash];
    }
}
