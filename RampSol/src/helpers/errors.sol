// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

/**
 * @dev Error Library
 */

library Errors {
    error Asset__AlreadyAllowed(address asset);
    error Asset__NotPresent(address asset);
    error Asset__InvalidAddress(address asset);
    error Invalid__VaultAddress(address vault);
    error Invalid__FeePercentage(uint256 feePercentage);
    error Invalid__AssetBalance(address asset, address user);
    error Asset__TransferFailed(address asset, address user);
    error Math__AdditionError(uint256 a, uint256 b);
    error Math__SubtractionError(uint256 a, uint256 b);
    error Math__MultiplicationError(uint256 a, uint256 b);
    error Math__DivisionError(uint256 a, uint256 b);
    error Revenue__WithdrawFailed(address asset);
    error Invalid__EthValue(uint256 value);
}