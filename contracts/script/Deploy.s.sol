// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {TumorIntel} from "../src/Fibonacci.sol";

/// @title Deploy TumorIntel to Base Sepolia
/// @notice Deploys the ZK simulation verifier contract.
/// @dev Usage:
///   forge script contracts/script/Deploy.s.sol:DeployTumorIntel \
///     --rpc-url $BASE_SEPOLIA_RPC_URL \
///     --broadcast --verify \
///     -vvvv
contract DeployTumorIntel is Script {
    /// @notice SP1 Verifier Gateway on Base Sepolia.
    /// See: https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address constant SP1_VERIFIER_GATEWAY = 0x397A5f7f3dBd538f23DE225B51f532c34448dA9B;

    function run() public {
        // Read simulation verification key from env or use placeholder
        bytes32 vkey = vm.envOr("SIMULATION_VKEY", bytes32(0));

        vm.startBroadcast();

        TumorIntel tumorIntel = new TumorIntel(SP1_VERIFIER_GATEWAY, vkey);

        vm.stopBroadcast();

        console.log("TumorIntel deployed at:", address(tumorIntel));
        console.log("Verifier gateway:", SP1_VERIFIER_GATEWAY);
        console.log("Simulation vkey:", vm.toString(vkey));
    }
}
