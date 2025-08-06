// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {RampContract} from "../src/RampContract.sol";

contract RampContractScript is Script {
    RampContract public rampContract;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        rampContract = new RampContract();
        rampContract.initialize(address(1), payable(address(2)));

        vm.stopBroadcast();
    }
}
