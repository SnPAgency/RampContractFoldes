// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;
import { Script, console } from "../lib/stand/src/Script.sol";
import { RampContract } from "../src/RampContract.sol";
import {ITransparentUpgradeableProxy} from "../lib/openzeppelin-contracts/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";


contract UpgradeRamp is Script {

    function setUp() public {
    }

    function run() public {
        uint256 privateKey = vm.envUint("HEDERA_PRIVATE_KEY");

        vm.startBroadcast(privateKey);

        address proxyAddress = vm.envAddress("PROXY_HEDERA");
        RampContract rampContract = new RampContract();
        ITransparentUpgradeableProxy(proxyAddress).upgradeToAndCall(
            address(rampContract),
            ""
        );
    }

}

