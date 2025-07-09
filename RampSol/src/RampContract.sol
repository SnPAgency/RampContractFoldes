// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;
/**
    * @title RampContract
    * @dev This contract is a placeholder for the Ramp protocol implementation.
    * It will include functions and state variables related to the Ramp protocol.
    * The Ramp protocol is designed to facilitate the exchange of assets in a decentralized manner.
    * This contract will be expanded in future iterations to include the full functionality of the Ramp protocol.
    * The Ramp protocol aims to provide a secure and efficient way to exchange assets with minimal reliance on
    * centralized intermediaries.

 */


import {PausableUpgradeable} from "@openzeppelin/utils/PausableUpgradeable.sol";
import {Initializable} from "@openzeppelin/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/access/OwnableUpgradeable.sol";

contract RampContract is Initializable, PausableUpgradeable, OwnableUpgradeable {

    /**
     * @dev Errors
     */
    error Asset__AlreadyAllowed(address asset);
    error Asset__InvalidAddress(address asset);

    /**
     * @dev Events
     */
    event AssetAllowedAdded(address indexed asset);

    /**
     * @dev State Variables
     */
    
    // Allowed Assets
    mapping(address => bool) public allowedAssets;

    /**
     * @dev Functions
     */

    /**
     * @dev Initializes the contract.
     */
    function initialize(address _controller) public initializer {
        __Pausable_init();
        __Ownable_init(_controller);
    }
    /**
     * @dev Pause function.
     * This function is used to pause the contract, preventing any state changes.
     * Only the Controller can call this function.
     */
    function pause() external onlyOwner {
        _pause();
    }
    /**
     * @dev Unpause function.
     * This function is used to unpause the contract, allowing state changes again.
     * Only the Controller can call this function.
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * @dev Add an asset to the allowed assets list.
     * @param asset The address of the asset to be added.
     * @notice This function can only be called by the owner of the contract.
     */
    function addAllowedAsset(address asset) external onlyOwner {
        // Ensure the asset address is valid and not already allowed

        if (asset == address(0)) {
            revert Asset__InvalidAddress(asset);
        }

        // Check if the asset is already allowed
        if (!allowedAssets[asset]) {
            allowedAssets[asset] = true;
            emit AssetAllowedAdded(asset);
        }
        // If the asset is already allowed, revert the transaction
        revert Asset__AlreadyAllowed(asset);
    }
}