// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

import {Test, console} from "forge-std/Test.sol";
import "forge-std/Vm.sol";
import {RampContract} from "../src/RampContract.sol";
import {RampToken} from "../src/test_contract/TestToken.sol";
import {IRampContract} from "../src/IRampContract.sol";
import {Errors} from "../src/helpers/errors.sol";

contract RampContractTest is Test {
    RampContract public rampContract;
    RampToken public rampToken1;
    RampToken public rampToken2;

    address public controller;
    address public vault;
    address public nonOwner;

    function setUp() public {
        controller = vm.addr(1);
        vault = vm.addr(2);
        nonOwner = vm.addr(3);
        rampToken1 = new RampToken(controller);
        rampToken2 = new RampToken(controller);
        rampContract = new RampContract();
        rampContract.initialize(controller, payable(vault));
    }

    // Test pause function
    function test_pause() public {
        vm.startPrank(controller);
        rampContract.pause();
        assertEq(rampContract.paused(), true);
        vm.stopPrank();
    }

    //Test non owner cannot pause
    function test_non_owner_cannot_pause() public {
        vm.startPrank(nonOwner);
        vm.expectRevert();
        rampContract.pause();
        vm.stopPrank();
    }

    // Test unpause function
    function test_unpause() public {
        vm.startPrank(controller);
        rampContract.pause();
        assertEq(rampContract.paused(), true);
        rampContract.unpause();
        assertEq(rampContract.paused(), false);
        vm.stopPrank();
    }

    //Test non owner cannot unpause
    function test_non_owner_cannot_unpause() public {
        vm.startPrank(nonOwner);
        vm.expectRevert();
        rampContract.unpause();
        vm.stopPrank();
    }

    // Test add allowed asset fails when supplied with address 0
    function test_add_allowed_asset_fails_with_address_0() public {
        vm.startPrank(controller);
        vm.expectRevert(abi.encodeWithSelector(Errors.Asset__InvalidAddress.selector, address(0)));
        rampContract.addAllowedAsset(address(0), controller, 1);
        vm.stopPrank();
    }

    // Tests adding of an allowed asset
    function test_add_allowed_asset() public {
        vm.startPrank(controller);
        rampToken1.approve(address(rampContract), 100);
        vm.expectEmit(true, true, false, true);
        emit IRampContract.AssetAllowedAdded(address(rampToken1), controller, 1, 100);
        rampContract.addAllowedAsset(address(rampToken1), controller, 1);

        //check the asset is allowed on the contract
        assertEq(rampContract.isAssetAllowed(address(rampToken1)), true);

        //confirm the asset balance on the contract
        assertEq(rampToken1.balanceOf(address(rampContract)), 100);

        //confirm the fee percentage
        assertEq(rampContract.getAssetFeePercentage(address(rampToken1)), 1);
        
        vm.stopPrank();
    }

    // Test adding an asset with unallowed fee percentage range
    function test_add_allowed_asset_fails_with_invalid_fee() public {
        vm.startPrank(controller);
        vm.expectRevert(abi.encodeWithSelector(Errors.Invalid__FeePercentage.selector, 0));
        // add an asset with the fee as 0
        rampContract.addAllowedAsset(address(rampToken1), controller, 0);
        vm.stopPrank();
    }

    // Test adding an asset with unallowed fee percentage range
    function test_add_allowed_asset_fails_with_invalid_fee_percentage() public {
        vm.startPrank(controller);
        vm.expectRevert(abi.encodeWithSelector(Errors.Invalid__FeePercentage.selector, 6));
        // add an asset with the fee as 6
        rampContract.addAllowedAsset(address(rampToken1), controller, 6);
        vm.stopPrank();
    }

    //Test remove allowed asset
    function test_remove_allowed_asset() public {
        vm.startPrank(controller);

        // approve the contract to spend some tokens
        rampToken2.approve(address(rampContract), 100);
        // add the asset first
        rampContract.addAllowedAsset(address(rampToken2), controller, 1);

        //check the asset is added
        assertEq(rampContract.isAssetAllowed(address(rampToken2)), true);

        // check that the contract balance matches the amount approved
        assertEq(rampToken2.balanceOf(address(rampContract)), 100);

        //remove the asset
        rampContract.removeAllowedAsset(address(rampToken2), controller);
        assertEq(rampContract.isAssetAllowed(address(rampToken2)), false);

        // check that the contract balance is 0
        assertEq(rampToken2.balanceOf(address(rampContract)), 0);

        //confirm the fee percentage
        assertEq(rampContract.getAssetFeePercentage(address(rampToken2)), 0);
        vm.stopPrank();
    }

    //Test remove allowed asset with invalid address
    function test_remove_allowed_asset_fails_with_invalid_address() public {
        vm.startPrank(controller);

        //expect revert
        vm.expectRevert(abi.encodeWithSelector(Errors.Asset__InvalidAddress.selector, address(0)));
        rampContract.removeAllowedAsset(address(0), controller);
        vm.stopPrank();
    }

    //Test set new Vault address
    function test_set_new_vault_address() public {
        vm.startPrank(controller);
        address payable newVault = payable(vm.addr(10));
        rampContract.setNewVault(newVault);
        assertEq(rampContract.vault(), newVault);
        vm.stopPrank();
    }

    //Test set new Vault address with invalid address
    function test_set_new_vault_address_with_invalid_address() public {
        vm.startPrank(controller);
        vm.expectRevert(abi.encodeWithSelector(Errors.Invalid__VaultAddress.selector, address(0)));
        rampContract.setNewVault(payable(address(0)));
        vm.stopPrank();
    }

    // Test set new Vault address with same address
    function test_set_new_vault_address_with_same_address() public {
        vm.startPrank(controller);
        vm.expectRevert(abi.encodeWithSelector(Errors.Invalid__VaultAddress.selector, vault));
        rampContract.setNewVault(payable(vault));
        vm.stopPrank();
    }

    // Test non owner cannot set new vault address
    function test_non_owner_cannot_set_new_vault_address() public {
        vm.startPrank(nonOwner);
        vm.expectRevert();
        rampContract.setNewVault(payable(vault));
        vm.stopPrank();
    }
}
