// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Script, console } from "forge-std/Script.sol";
import { Config } from "forge-std/Config.sol";
import { Vm } from "forge-std/Vm.sol";
import { Token } from "../src/test_contract/TestToken.sol";
import {IERC20Metadata} from "@openzeppelin-contracts/interfaces/IERC20Metadata.sol";

interface IERC20 is IERC20Metadata {
    function mint(address to, uint256 amount) external;
}

contract MintTokenScript is Script, Config {

    function setUp() public {}

    function run() public {
        //string memory toml = vm.readFile("deployments.toml");

        _loadConfig("deployments.toml", true);
        
        address token_address = config.get("usdt").toAddress(); 

        uint256 mint_amount = config.get("mint_amount").toUint256();

        IERC20 token = IERC20(token_address);


        uint256 privateKey = config.get("multisig").toUint256();

        address controller = config.get("controler").toAddress();
        vm.startBroadcast(privateKey);

        token.mint(controller, mint_amount * 10 ** token.decimals());
        console.log("Token Address: ", address(token));
        vm.stopBroadcast();
    }
}