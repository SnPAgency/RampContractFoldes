// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import {Script, console} from "forge-std/Script.sol";
import {RampContract} from "../src/RampContract.sol";
import {Upgrades} from "../lib/openzeppelin-foundry-upgrades/src/Upgrades.sol";
import {Options} from "../lib/openzeppelin-foundry-upgrades/src/Options.sol";

contract RampHederaContractScript is Script {
    RampContract public rampContract;

    function setUp() public {}

    function run() public {

        uint256 privateKey = vm.envUint("HEDERA_PRIVATE_KEY");
        address controller = vm.envAddress("HEDERA_CONTROLLER");
        address payable vault = payable(vm.envAddress("HEDERA_VAULT"));

        vm.startBroadcast(privateKey);

        address proxyAddress = Upgrades.deployTransparentProxy(
            "RampContract.sol",
            controller,
            abi.encodeCall(RampContract.initialize, (controller, vault))

        );

        address implementationAddress = Upgrades.getImplementationAddress(proxyAddress);
        console.log("Implementation Address: ", implementationAddress);
        console.log("Proxy Address: ", proxyAddress);
        vm.stopBroadcast();
    }
}