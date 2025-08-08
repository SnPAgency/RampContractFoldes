// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

import {Test, console} from "forge-std/Test.sol";
import "forge-std/Vm.sol";
import {RampContract} from "../src/RampContract.sol";
import {RampToken} from "../src/test_contract/TestToken.sol";
import {IRampContract} from "../src/IRampContract.sol";
import {Errors} from "../src/helpers/errors.sol";

contract RampContractTestFunctionality is Test {
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
        rampToken1.mint(controller, 100);
        rampToken2.mint(controller, 100);
        rampToken1.mint(nonOwner,10000);
        rampToken2.mint(nonOwner,10000);
        vm.startPrank(controller);
        rampToken1.approve(address(rampContract), 100);
        rampContract.addAllowedAsset(address(rampToken1), controller, 1);
        rampContract.setNewFeePercentage(address(0), 1);
        vm.deal(controller, 100);
        rampContract.fundContractWithEth{value: 99}();
        vm.stopPrank();
    }

    function test_onramp_deposit() public {
        vm.startPrank(nonOwner);
        uint256 balance = rampToken1.balanceOf(address(rampContract));
        uint256 userBalance = rampToken1.balanceOf(nonOwner);
        rampToken1.approve(address(rampContract), 100);

        vm.expectEmit(true, true, false, true);
        (uint256 fee, uint256 amountAfterFee) = rampContract.amountAfterFees(1, 100);
        emit IRampContract.RampDeposit(
            address(rampToken1),
            nonOwner,
            amountAfterFee,
            IRampContract.OnrampMedium.Primary,
            IRampContract.Region.KEN,
            ""
        );
        rampContract.onRampDeposit(
            address(rampToken1),
            100,
            nonOwner,
            IRampContract.OnrampMedium.Primary,
            IRampContract.Region.KEN,
            ""
        );
        uint256 newRevenue = rampContract.getAssetRevenue(address(rampToken1));
        assertEq(newRevenue, fee);
        assertEq(rampToken1.balanceOf(address(rampContract)), balance + 100);
        assertEq(rampToken1.balanceOf(nonOwner), userBalance - 100);
        vm.stopPrank();
    }

    function test_onramp_native() public {
        vm.startPrank(nonOwner);
        uint256 balance = address(rampContract).balance;
        vm.deal(nonOwner, 100);
        uint256 userBalance = nonOwner.balance;
        assertEq(userBalance, 100);

        (uint256 fee, uint256 amountAfterFee) = rampContract.amountAfterFees(1, 90);
        vm.expectEmit(true, true, false, true);
        emit IRampContract.RampDeposit(
            address(address(0)),
            nonOwner,
            amountAfterFee,
            IRampContract.OnrampMedium.Primary,
            IRampContract.Region.KEN,
            ""
        );
        rampContract.onRampNative{value: 90}(
            IRampContract.OnrampMedium.Primary,
            IRampContract.Region.KEN,
            ""  
        );

        uint256 newRevenue = rampContract.getAssetRevenue(address(0));
        assertEq(newRevenue, fee);
        assertEq(address(rampContract).balance, balance + 90);
        assertEq(nonOwner.balance, userBalance - 90);
        vm.stopPrank();
    }


    function test_onramp_with_permit() public {
        vm.startPrank(nonOwner);
        uint256 balance = rampToken1.balanceOf(address(rampContract));
        uint256 userBalance = rampToken1.balanceOf(nonOwner);

        (uint256 fee, uint256 amountAfterFee) = rampContract.amountAfterFees(1, 100);

        bytes32 domain_separator = rampToken1.DOMAIN_SEPARATOR();
        bytes32 PERMIT_TYPEHASH = keccak256(
            "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"
        );
        bytes32 struct_hash = keccak256(
            abi.encode(
                PERMIT_TYPEHASH,
                nonOwner,
                address(rampContract),
                100,
                rampToken1.nonces(nonOwner),
                block.timestamp
            )
        );
        bytes32 digest = keccak256(abi.encodePacked(
            "\x19\x01",
            domain_separator,
            struct_hash
        ));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(3, digest);

        vm.stopPrank();
        vm.startPrank(controller);
        vm.expectEmit(true, true, false, true);
        emit IRampContract.RampDeposit(
            address(rampToken1),
            nonOwner,
            amountAfterFee,
            IRampContract.OnrampMedium.Primary,
            IRampContract.Region.KEN,
            ""
        );
        rampContract.onRampWithPermit(
            address(rampToken1),
            100,
            nonOwner,
            IRampContract.PermitParams({
                deadline: block.timestamp,
                v: v,
                r: r,
                s: s
            }),
            IRampContract.OnrampMedium.Primary,
            IRampContract.Region.KEN,
            ""
        );
        uint256 newRevenue = rampContract.getAssetRevenue(address(rampToken1));
        console.log(newRevenue);
        console.log(fee);
        assertEq(newRevenue, fee);
        assertEq(rampToken1.balanceOf(address(rampContract)), balance + 100);
        assertEq(rampToken1.balanceOf(nonOwner), userBalance - 100);
        vm.stopPrank();
    }


    function test_offram_withdraw() public {
        vm.startPrank(nonOwner);
        uint256 balance = rampToken1.balanceOf(address(rampContract));
        uint256 userBalance = rampToken1.balanceOf(nonOwner);

        vm.stopPrank();
        vm.startPrank(controller);
        vm.expectEmit(true, true, false, true);
        emit IRampContract.RampWithdraw(
            address(rampToken1),
            nonOwner,
            10
        );
        rampContract.offRampWithdraw(
            address(rampToken1),
            10,
            nonOwner
        );
        assertEq(rampToken1.balanceOf(address(rampContract)), balance - 10);
        assertEq(rampToken1.balanceOf(nonOwner), userBalance + 10);
        vm.stopPrank();
    }

    function test_offram_native() public {
        vm.startPrank(controller);
        uint256 balance = address(rampContract).balance;
        uint256 userBalance = nonOwner.balance;

        vm.stopPrank();
        vm.startPrank(controller);
        vm.expectEmit(true, true, false, true);
        emit IRampContract.RampWithdraw(
            address(address(0)),
            nonOwner,
            10
        );
        rampContract.offRampNative(
            payable(nonOwner),
            10
        );
        assertEq(address(rampContract).balance, balance - 10);
        assertEq(nonOwner.balance, userBalance + 10);
        vm.stopPrank();
    }
}
