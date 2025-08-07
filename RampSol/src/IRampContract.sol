// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;



interface IRampContract {

        /**
     * @dev DataStructures
     */

    struct AssetInfo {
        // onchain revenue from onramp operations
        uint256 feePercentage;
        // onchain revenue from onramp operations
        uint256 revenuePerAsset;
    }

    struct PermitParams {
        // deadline for the permit
        uint256 deadline;
        // v value of the signature
        uint8 v;
        // r value of the signature
        bytes32 r;
        // s value of the signature
        bytes32 s;
    }

    /**
     * @dev Events
     */

    // Event emitted when an asset is added to the allowed assets list
    event AssetAllowedAdded(
        address indexed asset,
        address indexed funder,
        uint256 initialFeePercentage,
        uint256 initialBalance

    );

    // Event emitted when an asset is removed from the allowed assets list
    event AssetAllowedRemoved(
        address indexed asset,
        address indexed balanceRecipient,
        uint256 balance
    );

    // Event emitted when a deposit is made to the Ramp protocol
    event RampDeposit(
        address indexed asset,
        address indexed sender,
        uint256 amount,
        IRampContract.OnrampMedium medium,
        IRampContract.Region region,
        bytes data
    );

    event EthReceived(address indexed sender, uint256 amount);

    // Event emitted when a withdrawal is made from the Ramp protocol
    event RampWithdraw(address indexed asset, address indexed recipient, uint256 amount);

    // Event emitted when Ether is withdrawn from the contract
    event EthWithdrawn(address indexed recipient, uint256 amount);

    // Event emitted when the vault address is changed
    event VaultChanged(address indexed oldVault, address indexed newVault);

    event AssetRevenueWithdrawn(address indexed asset, uint256 amount);


    // Onramp Medium
    // Defines where ofchain funds should be sent
    //ie medium: Safaricom, Airtel, and PayStack
    enum OnrampMedium {
        Primary,
        Secondary,
        Tertiary
    }

    // Region
    enum Region {
        KEN,
        RWN,
        NGA,
        SA,
        EGY,
        GHN
    }

    function addAllowedAsset(
        address asset,
        address funder,
        uint256 feePercentage
    ) external;

    function removeAllowedAsset(
        address asset,
        address balanceRecipient
    ) external;

    function onRampDeposit(
        address asset,
        uint256 amount,
        address sender,
        OnrampMedium medium,
        Region region,
        bytes memory data
    ) external;

    function onRampNative(
        OnrampMedium medium,
        Region region,
        bytes memory data
    ) external payable;
    
    function offRampWithdraw(
        address asset,
        uint256 amount,
        address recipient
    ) external;

    function offRampNative(address payable receiver, uint256 amount) external;

    function onRampWithPermit(
        address asset,
        uint256 amount,
        address sender,
        PermitParams memory permitParams
    ) external ;
    function withdrawAssetRevenue(address asset) external;
    function withdrawEtherRevenue(uint256 amount) external;
    function setNewVault(address payable newVault) external;
    function setNewFeePercentage(address asset, uint256 feePercentage) external;
    function getAssetRevenue(address asset) external view returns (uint256);
    function getAssetFeePercentage(address asset) external view returns (uint256);
    function getAllowedAssets() external view returns (address[] memory);
    function getAssetInfo(address asset) external view returns (AssetInfo memory);
    function isAssetAllowed(address asset) external view returns (bool);
    function vault() external view returns (address payable);
}
