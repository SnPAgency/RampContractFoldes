// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import {Script, console} from "../lib/stand/src/Script.sol";
import {Token} from "../src/test_contract/TestToken.sol";

contract HederaTokenScript is Script {
    Token public token;

    function setUp() public {}

    function run() public {

        uint256 privateKey = vm.envUint("HEDERA_PRIVATE_KEY");
        address controller = vm.envAddress("HEDERA_CONTROLER");
        vm.startBroadcast(privateKey);

        token = new Token("USDC Coin", "USDC", controller);
        console.log("Token Address: ", address(token));
        vm.stopBroadcast();
    }
}