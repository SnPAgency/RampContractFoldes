module RampSup::ramp {
    use std::error;
    use std::signer;
    use aptos_std::table;
    use supra_framework::object;
    use supra_framework::event;

    #[test_only]
    use std::debug;

    /// Errors
    /// These are used to handle errors in the contract.

    /// Error code for the contract state not found
    const ENO_CONTRACT_STATE: u64 = 0;
    /// Error code for the wrong contract owner as signer
    const ENO_OWNER: u64 = 1;
    /// Error code for the asset not found in the allowed assets table
    const ENO_ASSET: u64 = 2;
    /// Error code for the asset already exists in the allowed assets table
    const EASSET_EXISTS: u64 = 3;

    /**
     * Global Storage
    */

    // storage object name
    const RAMP_SUP: vector<u8> = b"SUPRA_RAMP_GLOBAL_STORAGE";


    #[resource_group_member(group = supra_framework::object::ObjectGroup)]
    /// Global Storage
    /// This resource holds the global state of the RampSup contract.
    struct GlobalStorage has key {
        /// Indicates if the contract is active or not
        is_active: bool,
        /// The address of the owner of the contract
        /// This address is used to manage the contract and its assets and permissions
        /// The owner can add or remove assets, change the contract state, and change the owner
        owner: address,
        /// A table that contains the allowed assets for the contract
        allowed_assets: table::Table<address, bool>,
   }


    #[event]
    /// Event emitted when an asset is added
    struct AssetAddedEvent has store, drop {
        asset_address: address,
    }

    #[event]
    /// Event emitted when an asset is removed
    struct AssetRemovedEvent has store, drop {
        asset_address: address,
    }

    #[event]
    /// Event emitted when the contract state is changed
    struct ContractStateChangedEvent has store, drop {
        new_state: bool,
    }

    #[event]
    /// Event emitted when the owner is changed
    struct OwnerChangedEvent has store, drop {
        new_owner: address,
    }

    public entry fun initialize(owner: &signer, admin: address) {
        // create a global storage object to house all global storage
        let constructor_ref = object::create_named_object(owner, RAMP_SUP);

        // generate a signer for the global storage object
        let global_signer = &object::generate_signer(&constructor_ref);

        move_to(global_signer, GlobalStorage {
            is_active: true,
            owner: admin,
            allowed_assets: table::new<address, bool>(),
        });
    }


    // function: get_obj_address
    // # Notice
    //  - Returns the address of the global storage object
    //  - This function is used to get the address of the global storage object
    fun get_obj_address(): address {
        object::create_object_address(&@RampSup, RAMP_SUP)
    }


    // function add_asset
    // Adds an asset to the allowed assets table
    // # Parameters
    //  - `owner`: The signer who is adding the asset
    //  - `asset`: The address of the asset to be added
    // # Notice
    //  - The contract owner must be the one who is adding the asset
    //  - The asset must not already be in the allowed assets table
    //  - Emits an AssetAddedEvent when the asset is added
    //  - This function acquires the GlobalStorage resource

    public entry fun add_asset(owner: &signer, asset: address) acquires GlobalStorage {
        let owner_addr = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(owner_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(owner_addr).owner == signer::address_of(owner),
            error::permission_denied(ENO_OWNER)
        );
        // Ensure the asset is not already in the allowed assets table
        assert!(!table::contains(
            &borrow_global<GlobalStorage>(owner_addr).allowed_assets, asset),
            error::already_exists(EASSET_EXISTS)
        );

        table::add(&mut borrow_global_mut<GlobalStorage>(owner_addr).allowed_assets, asset, true);
        event::emit(AssetAddedEvent { asset_address: asset });
    }

    // function: remove_asset
    // Removes an asset from the allowed assets table
    // # Parameters
    //  - `owner`: The signer who is removing the asset
    //  - `asset`: The address of the asset to be removed
    // # Notice
    //  - The contract owner must be the one who is removing the asset
    //  - The asset must be in the allowed assets table
    //  - Emits an AssetRemovedEvent when the asset is removed
    //  - This function acquires the GlobalStorage resource
    public entry fun remove_asset(owner: &signer, asset: address) acquires GlobalStorage {
        let obj_addr = get_obj_address();

        let owner_addr = signer::address_of(owner);
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to remove the asset
        assert!(borrow_global<GlobalStorage>(
            obj_addr).owner == owner_addr,
            error::permission_denied(ENO_OWNER)
        );
        // Ensure the asset is in the allowed assets table
        assert!(table::contains(
            &borrow_global<GlobalStorage>(obj_addr).allowed_assets, asset),
            error::not_found(ENO_ASSET)
        );
        // Remove the asset from the allowed assets table
        table::remove(&mut borrow_global_mut<GlobalStorage>(obj_addr).allowed_assets, asset);
        event::emit(AssetRemovedEvent { asset_address: asset });
    }

    // function: set_contract_state
    // Sets the contract state to active or inactive
    // # Parameters
    //  - `owner`: The signer who is changing the contract state
    //  - `state`: The new state of the contract (true for active, false for inactive)
    // # Notice
    //  - The contract owner must be the one who is changing the state
    //  - Emits a ContractStateChangedEvent when the state is changed
    //  - This function acquires the GlobalStorage resource
    public entry fun set_contract_state(owner: &signer, state: bool) acquires GlobalStorage {
        // Get the address of the global storage object
        let obj_addr = get_obj_address();

        let owner_addr = signer::address_of(owner);
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to change the state
        assert!(borrow_global<GlobalStorage>(
            obj_addr).owner == owner_addr,
            error::permission_denied(ENO_OWNER)
        );
        // Set the contract state
        borrow_global_mut<GlobalStorage>(obj_addr).is_active = state;
        event::emit(ContractStateChangedEvent { new_state: state });
    }

    // function: set_owner
    // # Notice
    //  - Sets a new owner for the contract
    // # Parameters
    //  - `owner`: The signer who is changing the owner
    //  - `new_owner`: The address of the new owner
    // # Notice
    //  - The contract owner must be the one who is changing the owner
    //  - The new owner must not be the same as the current owner
    //  - Emits an OwnerChangedEvent when the owner is changed
    //  - This function acquires the GlobalStorage resource
    public entry fun set_owner(owner: &signer, new_owner: address) acquires GlobalStorage {
        // Get the address of the global storage object
        let obj_address = get_obj_address();
        let owner_addr = signer::address_of(owner);
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to change the owner
        assert!(borrow_global<GlobalStorage>(
            obj_address).owner == owner_addr,
            error::permission_denied(ENO_OWNER)
        );
        // Ensure the new owner is not the old owner
        assert!(new_owner != owner_addr, error::permission_denied(ENO_OWNER));
        // Set the new owner
        borrow_global_mut<GlobalStorage>(obj_address).owner = new_owner;

        event::emit(OwnerChangedEvent { new_owner });
    }

    // function is_active
    // # Notice
    //  - Checks if the contract is active or not
    //  - This function acquires the GlobalStorage resource

    #[view]
    public fun is_active(): bool acquires GlobalStorage {
        let obj_address = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_CONTRACT_STATE));
        borrow_global<GlobalStorage>(obj_address).is_active
    }

    // function: get_owner
    // # Notice
    //  - Gets the owner address of the contract
    //  - This function acquires the GlobalStorage resource
    #[view]
    public fun get_owner(): address acquires GlobalStorage {
        let obj_address = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_OWNER));
        // Return the owner address
        borrow_global<GlobalStorage>(obj_address).owner
    }

    // function: is_asset_allowed
    // # Notice
    //  - Checks if an asset is allowed in the contract
    // # Parameters
    //  - `asset`: The address of the asset to be checked
    // # Notice
    //  - This function acquires the GlobalStorage resource
    #[view]
    public fun is_asset_allowed(asset: address): bool acquires GlobalStorage {
        let obj_address = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_ASSET));
        // Check if the asset is in the allowed assets table
        table::contains(&borrow_global<GlobalStorage>(obj_address).allowed_assets, asset)
    }

    #[view]
    public fun get_global_storage(): (bool, address) acquires GlobalStorage {
        let global_storage = borrow_global<GlobalStorage>(get_obj_address());
        (global_storage.is_active, global_storage.owner)
    
    }


    #[test_only(owner = @RampSup, admin = @0x2)]
    public entry fun initialize_test(owner: signer, admin: address) {
        let msg: std::string::String = std::string::utf8(b"Running test for initialize...");
        debug::print(&msg);
        initialize(&owner, admin);
    }

    #[test(owner = @RampSup, admin= @0x2, asset = @0xCAFE)]
    public entry fun test_add_asset(owner: signer, admin: signer, asset: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for add_asset...");
        debug::print(&msg);

        supra_framework::account::create_account_for_test(signer::address_of(&owner));
        initialize(&owner, signer::address_of(&admin));
        add_asset(&admin, asset);

        assert!(event::was_event_emitted(
            &AssetAddedEvent { asset_address: asset }
        ), 4);
        assert!(is_asset_allowed(asset), 1);
    }

    #[test(owner = @RampSup, admin= @0x2, asset = @0xCAFE)]
    public entry fun test_remove_asset(owner: signer, admin: signer, asset: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for remove_asset...");
        debug::print(&msg);

        supra_framework::account::create_account_for_test(signer::address_of(&owner));
        initialize(&owner, signer::address_of(&admin));
        add_asset(&admin, asset);
        remove_asset(&admin, asset);

        assert!(event::was_event_emitted(
            &AssetRemovedEvent { asset_address: asset }
        ), 4);
        assert!(!is_asset_allowed(asset), 5);
    }

    #[test(owner = @RampSup, admin= @0x2)]
    public entry fun test_set_contract_state(owner: signer, admin: signer) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for set_contract_state...");
        debug::print(&msg);
        supra_framework::account::create_account_for_test(signer::address_of(&owner));
        initialize(&owner, signer::address_of(&admin));
        set_contract_state(&admin, false);
        assert!(is_active() == false, 1);
        assert!(event::was_event_emitted(
            &ContractStateChangedEvent { new_state: false }
        ), 2);
    }


    #[test(owner = @RampSup, admin= @0x2, new_owner = @0x3)]
    public entry fun test_set_owner(owner: signer, admin: signer, new_owner: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for set_owner...");
        debug::print(&msg);
        supra_framework::account::create_account_for_test(signer::address_of(&owner));
        initialize(&owner, signer::address_of(&admin));
        set_owner(&admin, new_owner);
        assert!(get_owner() == new_owner, 1);
        assert!(event::was_event_emitted(
            &OwnerChangedEvent { new_owner }
        ), 2);
    }

}
