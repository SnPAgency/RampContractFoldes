// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import {Script, console} from "forge-std/Script.sol";
import {Token} from "../src/test_contract/TestToken.sol";

contract TokenScript is Script {
    Token public token;

    function setUp() public {}

    function run() public {

        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address controller = vm.envAddress("CONTROLER");
        vm.startBroadcast(privateKey);

        token = new Token("USDT Coin", "USDT", controller);
        console.log("Token Address: ", address(token));
        vm.stopBroadcast();
    }
}