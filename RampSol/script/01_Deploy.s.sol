// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import {Script, console} from "forge-std/Script.sol";
import {RampContract} from "../src/RampContract.sol";
import {Upgrades} from "openzeppelin-foundry-upgrades/Upgrades.sol";
import {Options} from "openzeppelin-foundry-upgrades/Options.sol";
//import {Errors} from "../src/helpers/errors.sol";

contract RampContractScript is Script {
    RampContract public rampContract;

    function setUp() public {}

    function run() public {

        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address controller = vm.envAddress("CONTROLER");
        address payable vault = payable(vm.envAddress("VAULT"));

        vm.startBroadcast(privateKey);

        //Errors errorLib = new Errors();

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