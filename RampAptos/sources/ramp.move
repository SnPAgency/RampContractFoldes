module RampAptos::ramp {
    use std::error;
    use std::signer;
    use aptos_std::simple_map;
    use aptos_framework::object::{Self, Object, ExtendRef};
    use aptos_framework::event;
    use aptos_framework::fungible_asset::{Self, Metadata, FungibleStore};
    use aptos_framework::primary_fungible_store::{ensure_primary_store_exists, deposit};
    use aptos_framework::option;
    use std::string;

    #[test_only]
    use std::debug;

    /// Errors
    /// These are used to handle errors in the contract.

    /// Error code for the contract state not found
    const ENO_CONTRACT_STATE: u64 = 0;
    /// Error code for the wrong contract owner as signer
    const ENOT_OWNER: u64 = 1;
    /// Error code for the asset not found in the allowed assets simple_map
    const ENO_ASSET: u64 = 2;
    /// Error code for the asset already exists in the allowed assets simple_map
    const EASSET_EXISTS: u64 = 3;
    /// Error code for assets not allowed
    const EASSET_NOT_ALLOWED: u64 = 4;
    /// Error code for insufficient asset balance in contract
    const EINSUFICIENT_ASSET_BALANCE: u64 = 5; 

    /// global storage name
    const RAMP_APTOS: vector<u8> = b"RAMP_APTOS_GLOBAL_STORAGE";

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Global Storage
    /// This resource holds the global state of the RampAptos contract.
    struct GlobalStorage has key {
    
        /// Indicates if the contract is active or not
        is_active: bool,
    
        /// The address of the owner of the contract
        /// This address is used to manage the contract and its assets and permissions
        /// The owner can add or remove assets, change the contract state, and change the owner
        owner: address,

        /// vault address
        /// address where assets are stored
        vault_address: simple_map::SimpleMap<Object<Metadata>, VaultStore>,
   }

    /// Vault where assets are stored
    /// also includes the assets's fee and fa extend ref
    struct VaultStore has store, drop {
        /// Fungible asset store
        store: Object<FungibleStore>,

        /// Fee Percentage
        /// This variable holds the percentage of the fee charged for each asset.
        asset_fee_percentage: u64,

        /// extend ref to enable asset transfer from store
        vault_extend_ref: ExtendRef,

        /// asset revenue
        asset_revenue: u64
    }

    #[event]
    /// Event emitted when an asset is added
    struct AssetAddedEvent has store, drop {
        asset_address: Object<Metadata>,
    }

    #[event]
    /// Event emitted when an asset is removed
    struct AssetRemovedEvent has store, drop {
        asset_address: Object<Metadata>,
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

    #[event]
    /// Event emitted when a deposit is made to the Ramp protocol

    struct RampDeposit has store, drop {
        asset: Object<Metadata>,
        amount: u64,
        sender: address
    }

    #[event]
    /// Event emitted when a withdrawal is made from the Ramp protocol
    struct RampWithdraw has store, drop {
        asset: Object<Metadata>,
        amount: u64,
        recipient: address,
    }

    #[event]
    /// Event emitted when the vault address is changed
    struct VaultChanged has store, drop {
        old_vault: Object<Metadata>,
        new_vault: address
    }

    #[event]
    /// event emitted when the fee to an asset gets changed
    struct AssetFeeChanged has store, drop {
        asset: Object<Metadata>,
        new_fee: u64
    }



    public entry fun initialize(owner: &signer, admin: address) {
        let constructor_ref = object::create_named_object(owner, RAMP_APTOS);
        let global_signer = &object::generate_signer(&constructor_ref);
        
        move_to(global_signer, GlobalStorage {
            is_active: true,
            owner: admin,
            //allowed_assets: simple_map::new<Object<Metadata>, bool>(),
            vault_address: simple_map::new<Object<Metadata>, VaultStore>(),
            //project_revenue_per_asset: simple_map::new<Object<Metadata>, u64>()
        });
    }

    // function: get_obj_address
    // # Notice
    //  - Returns the address of the global storage object
    //  - This function is used to get the address of the global storage object
    fun get_obj_address(): address {
        object::create_object_address(&@RampAptos, RAMP_APTOS)
    }

    // function add_asset
    // Adds an asset to the allowed assets simple_map
    // # Parameters
    //  - `owner`: The signer who is adding the asset
    //  - `asset`: The address of the asset to be added
    // # Notice
    //  - The contract owner must be the one who is adding the asset
    //  - The asset must not already be in the allowed assets simple_map
    //  - Emits an AssetAddedEvent when the asset is added
    //  - This function acquires the GlobalStorage resource

    public entry fun add_asset(owner: &signer, asset: Object<Metadata>, fee: u64) acquires GlobalStorage {
        let owner_addr = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(owner_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(owner_addr).owner == signer::address_of(owner),
            error::permission_denied(ENOT_OWNER)
        );

        let global_storage = borrow_global_mut<GlobalStorage>(owner_addr);

        let store_constructor_ref = &object::create_object(owner_addr);

        let fa_store = fungible_asset::create_store(store_constructor_ref, asset);

        simple_map::upsert(
            &mut global_storage.vault_address,
            asset,
            VaultStore {
                store: fa_store,
                asset_fee_percentage: fee,
                vault_extend_ref: object::generate_extend_ref(store_constructor_ref),
                asset_revenue: 0u64
            }
        );
    
        event::emit(AssetAddedEvent { asset_address: asset });
    }

    // function: remove_asset
    // Removes an asset from the allowed assets simple_map
    // # Parameters
    //  - `owner`: The signer who is removing the asset
    //  - `asset`: The address of the asset to be removed
    // # Notice
    //  - The contract owner must be the one who is removing the asset
    //  - The asset must be in the allowed assets simple_map
    //  - Emits an AssetRemovedEvent when the asset is removed
    //  - This function acquires the GlobalStorage resource
    public entry fun remove_asset(owner: &signer, asset: Object<Metadata>, asset_recipient: address) acquires GlobalStorage {
        let obj_addr = get_obj_address();

        let owner_addr = signer::address_of(owner);
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to remove the asset
        let global_storage = borrow_global<GlobalStorage>(obj_addr);

        assert!(borrow_global<GlobalStorage>(
            obj_addr).owner == owner_addr,
            error::permission_denied(ENOT_OWNER)
        );

        // Ensure the asset is in the allowed assets simple_map
        assert!(simple_map::contains_key(&global_storage.vault_address, &asset),
            error::not_found(ENO_ASSET)
        );
    
        let vault = simple_map::borrow(&global_storage.vault_address, &asset);
        let balance = fungible_asset::balance(vault.store);


        //// transfer all fa to another address
        if (balance > 0) {
            let store_signer = &object::generate_signer_for_extending(&vault.vault_extend_ref);
            let fa = fungible_asset::withdraw(store_signer, vault.store, balance);

            deposit(
                asset_recipient,
                fa
            );

        };
        // Remove the asset from the allowed assets simple_map
        simple_map::remove(&mut borrow_global_mut<GlobalStorage>(obj_addr).vault_address, &asset);
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
            error::permission_denied(ENOT_OWNER)
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
            error::permission_denied(ENOT_OWNER)
        );
        // Ensure the new owner is not the old owner
        assert!(new_owner != owner_addr, error::permission_denied(ENOT_OWNER));
        // Set the new owner
        borrow_global_mut<GlobalStorage>(obj_address).owner = new_owner;

        event::emit(OwnerChangedEvent { new_owner });
    }


    /// Deposit a fungible asset into the asset's vault
    public entry fun on_ramp_deposit(user: &signer, asset: Object<Metadata>, amount: u64) acquires GlobalStorage {
        let obj_address = get_obj_address();
        let user_store = ensure_primary_store_exists(signer::address_of(user), asset);

        let fungible_asset = fungible_asset::withdraw(
            user,
            user_store,
            amount
        );

        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_CONTRACT_STATE));

        // get global storage reference
        let global_storage_ref = borrow_global<GlobalStorage>(obj_address);

        // assert the asset is listed
        assert!(
            simple_map::contains_key(&global_storage_ref.vault_address, &asset),
            error::invalid_state(EASSET_NOT_ALLOWED)
        );
        // get the asset vault
        let asset_vault = simple_map::borrow(&global_storage_ref.vault_address, &asset);

        let amount = fungible_asset::amount(&fungible_asset);
        // deposit the fa to the store
        fungible_asset::deposit(asset_vault.store, fungible_asset);

        // emit the depist event
        event::emit(RampDeposit {
            asset: asset,
            amount: amount,
            sender: signer::address_of(user)
        });
    }


    /// Withdraw an asset for the vault
    /// only called by the owner of the module
    public entry fun off_ramp_withdraw(admin: &signer, asset: Object<Metadata>, recipient: address, amount: u64) acquires GlobalStorage {
        
        let obj_address = get_obj_address();
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(obj_address).owner == signer::address_of(admin),
            error::permission_denied(ENOT_OWNER)
        );
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_CONTRACT_STATE));

        //retrive the global storage ref
        let global_storage_ref = borrow_global<GlobalStorage>(obj_address);

        //ensure the asset is listed
        assert!(
            simple_map::contains_key(&global_storage_ref.vault_address, &asset),
            error::invalid_state(EASSET_NOT_ALLOWED)
        );

        // get the asset vault in storage
        let asset_vault = simple_map::borrow(&global_storage_ref.vault_address, &asset);

        // ensure the asset balance in vault is enough
        // @phydy: check later with the project comissions
        assert!(
            fungible_asset::balance(asset_vault.store) > amount,
            error::invalid_argument(EINSUFICIENT_ASSET_BALANCE)
        );

        // get the store signer
        let store_signer = &object::generate_signer_for_extending(&asset_vault.vault_extend_ref);

        let fa = fungible_asset::withdraw(store_signer, asset_vault.store, amount);

        //transfer asset to the recipient
        deposit(
            recipient,
            fa
        );
        // emit the Withdraw event
        event::emit(
            RampWithdraw {
                asset: asset,
                amount: amount,
                recipient: recipient
            }
        );
    }

    /// Change the fee percentage charged on an asset
    public entry fun set_fee(admin: &signer, asset: Object<Metadata>, new_fee: u64) acquires GlobalStorage {
        let obj_address = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to change fee
        assert!(
            borrow_global<GlobalStorage>(obj_address).owner == signer::address_of(admin),
            error::permission_denied(ENOT_OWNER)
        );

        simple_map::borrow_mut(
            &mut borrow_global_mut<GlobalStorage>(obj_address).vault_address, &asset
        ).asset_fee_percentage = new_fee;

        event::emit(AssetFeeChanged {
            asset: asset,
            new_fee: new_fee
        });
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
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENOT_OWNER));
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
    public fun is_asset_allowed(asset: Object<Metadata>): bool acquires GlobalStorage {
        let obj_address = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_ASSET));
        // Check if the asset is in the allowed assets simple_map
        simple_map::contains_key(&borrow_global<GlobalStorage>(obj_address).vault_address, &asset)
    }

    #[view]
    public fun get_global_storage(): (bool, address) acquires GlobalStorage {
        let global_storage = borrow_global<GlobalStorage>(get_obj_address());
        (global_storage.is_active, global_storage.owner)
    }

    #[view]
    public fun get_allowed_assets(): vector<Object<Metadata>> acquires GlobalStorage {
        let global_storage = borrow_global<GlobalStorage>(get_obj_address());

        simple_map::keys(&global_storage.vault_address)

    }


    #[test_only(owner = @RampAptos, admin = @0x2)]
    public entry fun initialize_test(owner: signer, admin: address) {
        let msg: std::string::String = std::string::utf8(b"Running test for initialize...");
        debug::print(&msg);
        initialize(&owner, admin);
    }

    #[test(owner = @RampAptos, admin= @0x2)]
    public entry fun test_add_asset(owner: signer, admin: signer) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for add_asset...");
        debug::print(&msg);

        aptos_framework::account::create_account_for_test(signer::address_of(&owner));
        initialize(&owner, signer::address_of(&admin));

        // Create a test fungible asset using the proper pattern
        let constructor_ref = &object::create_named_object(&owner, b"TEST_ASSET");
        aptos_framework::primary_fungible_store::create_primary_store_enabled_fungible_asset(
            constructor_ref,
            aptos_framework::option::none(),
            std::string::utf8(b"Test Asset"),
            std::string::utf8(b"TEST"),
            8,
            std::string::utf8(b"http://example.com/favicon.ico"),
            std::string::utf8(b"http://example.com"),
        );
        let metadata = object::object_from_constructor_ref<fungible_asset::Metadata>(constructor_ref);
        add_asset(&admin, metadata, 1u64);

        assert!(event::was_event_emitted(
            &AssetAddedEvent { asset_address: metadata }
        ), 4);
        assert!(is_asset_allowed(metadata), 1);
    }

    #[test(owner = @RampAptos, admin= @0x2)]
    public entry fun test_remove_asset(owner: signer, admin: signer) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for remove_asset...");
        debug::print(&msg);

        aptos_framework::account::create_account_for_test(signer::address_of(&owner));
        aptos_framework::account::create_account_for_test(signer::address_of(&admin));

        initialize(&owner, signer::address_of(&admin));
        // Create a test fungible asset using the proper pattern
        let constructor_ref = &object::create_named_object(&owner, b"TEST_ASSET");
        aptos_framework::primary_fungible_store::create_primary_store_enabled_fungible_asset(
            constructor_ref,
            aptos_framework::option::none(),
            std::string::utf8(b"Test Asset"),
            std::string::utf8(b"TEST"),
            8,
            std::string::utf8(b"http://example.com/favicon.ico"),
            std::string::utf8(b"http://example.com"),
        );
        let metadata = object::object_from_constructor_ref<fungible_asset::Metadata>(constructor_ref);

        add_asset(&admin, metadata, 1u64);

        remove_asset(&admin, metadata, signer::address_of(&admin));

        assert!(event::was_event_emitted(
            &AssetRemovedEvent { asset_address: metadata }
        ), 4);
        assert!(!is_asset_allowed(metadata), 5);
    }

    #[test(owner = @RampAptos, admin= @0x2)]
    public entry fun test_set_contract_state(owner: signer, admin: signer) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for set_contract_state...");
        debug::print(&msg);
        aptos_framework::account::create_account_for_test(signer::address_of(&owner));
        initialize(&owner, signer::address_of(&admin));
        set_contract_state(&admin, false);
        assert!(is_active() == false, 1);
        assert!(event::was_event_emitted(
            &ContractStateChangedEvent { new_state: false }
        ), 2);
    }


    #[test(owner = @RampAptos, admin= @0x2, new_owner = @0x4)]
    public entry fun test_set_owner(owner: signer, admin: signer, new_owner: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for set_owner...");
        debug::print(&msg);
        aptos_framework::account::create_account_for_test(signer::address_of(&owner));
        initialize(&owner, signer::address_of(&admin));
        set_owner(&admin, new_owner);
        assert!(get_owner() == new_owner, 1);
        assert!(event::was_event_emitted(
            &OwnerChangedEvent { new_owner }
        ), 2);
    }
}