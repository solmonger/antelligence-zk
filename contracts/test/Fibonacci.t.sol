// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {TumorIntel, SimulationProof} from "../src/Fibonacci.sol";
import {SP1VerifierGateway} from "@sp1-contracts/SP1VerifierGateway.sol";

contract TumorIntelTest is Test {
    address verifier;
    TumorIntel public tumorIntel;
    bytes32 constant MOCK_VKEY = bytes32(uint256(1));

    function setUp() public {
        verifier = address(new SP1VerifierGateway(address(1)));
        tumorIntel = new TumorIntel(verifier, MOCK_VKEY);
    }

    function test_Constructor() public view {
        assertEq(tumorIntel.verifier(), verifier);
        assertEq(tumorIntel.simulationVKey(), MOCK_VKEY);
    }

    function test_VerifySimulation() public {
        SimulationProof memory proof = SimulationProof({
            configHash: bytes32(uint256(0xdeadbeef)),
            killRate: 4545,
            nanobotCount: 10,
            tumorRadius: 150,
            steps: 300,
            seedHash: bytes32(uint256(0xcafe)),
            valid: true
        });

        bytes memory publicValues = abi.encode(proof);
        bytes memory fakeProofBytes = new bytes(32);

        vm.mockCall(
            verifier,
            abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector),
            abi.encode(true)
        );

        SimulationProof memory result = tumorIntel.verifySimulation(publicValues, fakeProofBytes);

        assertEq(result.killRate, 4545);
        assertEq(result.nanobotCount, 10);
        assertEq(result.tumorRadius, 150);
        assertEq(result.steps, 300);
        assertTrue(result.valid);
        assertTrue(tumorIntel.isVerified(proof.configHash));
    }

    function test_RevertOnInvalidSimulation() public {
        SimulationProof memory proof = SimulationProof({
            configHash: bytes32(uint256(0xbad)),
            killRate: 0,
            nanobotCount: 0,
            tumorRadius: 0,
            steps: 0,
            seedHash: bytes32(0),
            valid: false
        });

        bytes memory publicValues = abi.encode(proof);
        bytes memory fakeProofBytes = new bytes(32);

        vm.mockCall(
            verifier,
            abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector),
            abi.encode(true)
        );

        vm.expectRevert("Simulation validation failed");
        tumorIntel.verifySimulation(publicValues, fakeProofBytes);
    }

    function test_UnverifiedConfig() public view {
        assertFalse(tumorIntel.isVerified(bytes32(uint256(0x123))));
    }

    function test_RevertOnBadProof() public {
        SimulationProof memory proof = SimulationProof({
            configHash: bytes32(uint256(0xdeadbeef)),
            killRate: 4545,
            nanobotCount: 10,
            tumorRadius: 150,
            steps: 300,
            seedHash: bytes32(uint256(0xcafe)),
            valid: true
        });

        bytes memory publicValues = abi.encode(proof);
        bytes memory badProof = new bytes(32);

        vm.expectRevert();
        tumorIntel.verifySimulation(publicValues, badProof);
    }
}
