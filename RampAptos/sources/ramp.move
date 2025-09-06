module RampAptos::ramp {
    use std::error;
    use std::signer;
    use aptos_std::simple_map;
    use aptos_framework::object::{Self, Object, ExtendRef};
    use aptos_framework::event;
    use aptos_framework::account;
    use aptos_framework::fungible_asset::{
        Self,
        Metadata,
        FungibleStore,
    };
    //use aptos_framework::aptos_coin::{Self, AptosCoin};
    use aptos_std::type_info;
    use std::event::EventHandle;
    use aptos_framework::coin;
    use aptos_framework::primary_fungible_store::{ensure_primary_store_exists, deposit};

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

        coin_vaults: simple_map::SimpleMap<address, CoinVault>,

        global_extend_ref: ExtendRef,
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

    struct CoinVault has store, drop {
        /// Coin store
        store_amount: u64,

        coin_revenue: u64,

        coin_fee_percentage: u64,
    }

    struct RampEventStore has key {
        deposit_event_handle: EventHandle<RampDeposit>,
        withdraw_event_handle: EventHandle<RampWithdraw>,
        asset_added_event_handle: EventHandle<AssetAddedEvent>,
        coin_added_event_handle: EventHandle<CoinAddedEvent>,
        coin_removed_event_handle: EventHandle<CoinRemovedEvent>,
        asset_removed_event_handle: EventHandle<AssetRemovedEvent>,
        contract_state_changed_event_handle: EventHandle<ContractStateChangedEvent>,
        owner_changed_event_handle: EventHandle<OwnerChangedEvent>,
        vault_changed_event_handle: EventHandle<VaultChanged>,
        asset_fee_changed_event_handle: EventHandle<AssetFeeChanged>,
        coin_onramp_event_handle: EventHandle<CoinOnrampEvent>,
        coin_offramp_event_handle: EventHandle<CoinOfframpEvent>,
    }

    #[event]
    /// Event emitted when an asset is added
    struct AssetAddedEvent has store, drop {
        asset_address: Object<Metadata>,
        fee_percentage: u64,
        initial_amount: u64
    }

    #[event]
    /// Event emitted when a coin is added
    struct CoinAddedEvent has store, drop {
        coin_address: address,
        amount: u64,
    }

    #[event]
    /// Event emitted when a coin is removed
    struct CoinRemovedEvent has store, drop {
        coin_address: address,
        amount: u64,
        recipient: address,
    }

    #[event]
    /// Event emitted when an asset is removed
    struct AssetRemovedEvent has store, drop {
        asset_address: Object<Metadata>,
        balance: u64,
        receiver: address
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

    #[event]
    /// Event emitted when a coin is added
    struct CoinOnrampEvent has store, drop {
        coin_address: address,
        amount: u64,
    }

    #[event]
    /// Event emitted when a coin is removed
    struct CoinOfframpEvent has store, drop {
        coin_address: address,
        amount: u64,
        recipient: address,
    }


    fun init_module(owner: &signer) {
        let constructor_ref = object::create_named_object(owner, RAMP_APTOS);
        let global_signer = &object::generate_signer(&constructor_ref);
        let extend_ref = object::generate_extend_ref(&constructor_ref);
        let admin = object::root_owner(object::object_from_constructor_ref<object::ObjectCore>(&constructor_ref));
        move_to(global_signer, GlobalStorage {
            is_active: true,
            owner: admin,
            vault_address: simple_map::new<Object<Metadata>, VaultStore>(),
            coin_vaults: simple_map::new<address, CoinVault>(),
            global_extend_ref: extend_ref,
        });

        move_to(global_signer, RampEventStore {
            deposit_event_handle: account::new_event_handle<RampDeposit>(global_signer),
            withdraw_event_handle: account::new_event_handle<RampWithdraw>(global_signer),
            asset_added_event_handle: account::new_event_handle<AssetAddedEvent>(global_signer),
            coin_added_event_handle: account::new_event_handle<CoinAddedEvent>(global_signer),
            coin_removed_event_handle: account::new_event_handle<CoinRemovedEvent>(global_signer),
            asset_removed_event_handle: account::new_event_handle<AssetRemovedEvent>(global_signer),
            contract_state_changed_event_handle: account::new_event_handle<ContractStateChangedEvent>(global_signer),
            owner_changed_event_handle: account::new_event_handle<OwnerChangedEvent>(global_signer),
            vault_changed_event_handle: account::new_event_handle<VaultChanged>(global_signer),
            asset_fee_changed_event_handle: account::new_event_handle<AssetFeeChanged>(global_signer),
            coin_onramp_event_handle: account::new_event_handle<CoinOnrampEvent>(global_signer),
            coin_offramp_event_handle: account::new_event_handle<CoinOfframpEvent>(global_signer),
        });
    }

    /// function: add_coin
    /// # Parameters
    ///  - `owner`: The signer who is adding the coin
    ///  - `fee`: The fee percentage for the coin
    ///  - `amount`: The amount of the coin to be added
    /// # Notice
    ///  - The contract owner must be the one who is adding the coin
    ///  - Emits a CoinAddedEvent when the coin is added
    ///  - This function acquires the GlobalStorage resource
    public entry fun add_coin<CoinType>(owner: &signer, fee: u64, amount: u64) acquires GlobalStorage, RampEventStore {
        let obj_addr = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(obj_addr).owner == signer::address_of(owner),
            error::permission_denied(ENOT_OWNER)
        );
        let global_storage = borrow_global_mut<GlobalStorage>(obj_addr);
        
        let coin_amount = coin::withdraw<CoinType>(owner, amount);


        simple_map::upsert(&mut global_storage.coin_vaults, get_coin_address<CoinType>(), CoinVault {
            store_amount: amount,
            coin_revenue: 0u64,
            coin_fee_percentage: fee,
        });

        coin::deposit<CoinType>(obj_addr, coin_amount);

        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_addr).coin_added_event_handle, CoinAddedEvent {
            coin_address: get_coin_address<CoinType>(),
            amount,
        });

    }

    /// function: remove_coin
    /// # Parameters
    ///  - `owner`: The signer who is removing the coin
    ///  - `recipient`: The address of the recipient of the coin
    /// # Notice
    ///  - The contract owner must be the one who is removing the coin
    ///  - The coin must be in the coin_vaults simple_map
    ///  - Emits a CoinRemovedEvent when the coin is removed
    ///  - This function acquires the GlobalStorage resource
    public entry fun remove_coin<CoinType>(owner: &signer, recipient: address) acquires GlobalStorage, RampEventStore {
        let obj_addr = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(obj_addr).owner == signer::address_of(owner),
            error::permission_denied(ENOT_OWNER)
        );
        let global_storage = borrow_global_mut<GlobalStorage>(obj_addr);
        simple_map::remove(&mut global_storage.coin_vaults, &get_coin_address<CoinType>());
        let coin_balance = coin::balance<CoinType>(obj_addr);
        let coin_amount = coin::withdraw<CoinType>(owner, coin_balance);
        coin::deposit<CoinType>(recipient, coin_amount);
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_addr).coin_removed_event_handle, CoinRemovedEvent {
            coin_address: get_coin_address<CoinType>(),
            amount: coin_balance,
            recipient,
        });
    }

    /// function: onramp_coin
    /// # Parameters
    ///  - `owner`: The signer who is onramping the coin
    ///  - `amount`: The amount of the coin to be onramped
    /// # Notice
    ///  - The contract owner must be the one who is onramping the coin
    ///  - The coin must be in the coin_vaults simple_map
    ///  - Emits a CoinAddedEvent when the coin is onramped
    ///  - This function acquires the GlobalStorage resource
    public entry fun onramp_coin<CoinType>(owner: &signer, amount: u64) acquires GlobalStorage, RampEventStore {
        let obj_addr = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(obj_addr).owner == signer::address_of(owner),
            error::permission_denied(ENOT_OWNER)
        );
        //let global_storage = borrow_global_mut<GlobalStorage>(obj_addr);
        let coin_amount = coin::withdraw<CoinType>(owner, amount);
        coin::deposit<CoinType>(obj_addr, coin_amount);
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_addr).coin_onramp_event_handle, CoinOnrampEvent {
            coin_address: get_coin_address<CoinType>(),
            amount,
        });
    }

    /// function: offramp_coin
    /// # Parameters
    ///  - `owner`: The signer who is offramping the coin
    ///  - `receiver`: The address of the receiver of the coin
    ///  - `amount`: The amount of the coin to be offramped
    /// # Notice
    ///  - The contract owner must be the one who is offramping the coin
    ///  - The coin must be in the coin_vaults simple_map
    ///  - Emits a CoinRemovedEvent when the coin is offramped
    ///  - This function acquires the GlobalStorage resource
    public entry fun offramp_coin<CoinType>(owner: &signer, receiver: address, amount: u64) acquires GlobalStorage, RampEventStore {
        let obj_addr = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(obj_addr).owner == signer::address_of(owner),
            error::permission_denied(ENOT_OWNER)
        );
        let global_storage = borrow_global_mut<GlobalStorage>(obj_addr);
        let coin_balance = coin::balance<CoinType>(obj_addr);

        let obj_signer = &object::generate_signer_for_extending(&global_storage.global_extend_ref);
        let coin_amount = coin::withdraw<CoinType>(obj_signer, coin_balance);
        coin::deposit<CoinType>(receiver, coin_amount);
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_addr).coin_offramp_event_handle, CoinOfframpEvent {
            coin_address: get_coin_address<CoinType>(),
            amount,
            recipient: receiver,
        });
    }

    // function: get_obj_address
    // # Notice
    //  - Returns the address of the global storage object
    //  - This function is used to get the address of the global storage object
    fun get_obj_address(): address {
        object::create_object_address(&@RampAptos, RAMP_APTOS)
    }

    fun get_coin_address<CoinType>(): address {
        type_info::account_address(&type_info::type_of<CoinType>())
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

    public entry fun add_asset(owner: &signer, asset: Object<Metadata>, fee: u64, initial_amount: u64) acquires GlobalStorage, RampEventStore {
        let obj_addr = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_addr), error::not_found(ENO_CONTRACT_STATE));
        // Ensure the owner is the one who is trying to add the asset
        assert!(
            borrow_global<GlobalStorage>(obj_addr).owner == signer::address_of(owner),
            error::permission_denied(ENOT_OWNER)
        );

        let owner_store = ensure_primary_store_exists(signer::address_of(owner), asset);//aptos_framework::primary_fungible_store::primary_store(signer::address_of(owner), asset);

        let global_storage = borrow_global_mut<GlobalStorage>(obj_addr);

        let store_constructor_ref = &object::create_object(obj_addr);

        let fa_store = fungible_asset::create_store(store_constructor_ref, asset);

        fungible_asset::transfer(owner, owner_store, fa_store, initial_amount);
        
        simple_map::add(
            &mut global_storage.vault_address,
            asset,
            VaultStore {
                store: fa_store,
                asset_fee_percentage: fee,
                vault_extend_ref: object::generate_extend_ref(store_constructor_ref),
                asset_revenue: 0u64
            }
        );
    
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_addr).asset_added_event_handle, AssetAddedEvent {
            asset_address: asset,
            fee_percentage: fee,
            initial_amount: initial_amount
        });
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
    public entry fun remove_asset(owner: &signer, asset: Object<Metadata>, asset_recipient: address) acquires GlobalStorage, RampEventStore {
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
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_addr).asset_removed_event_handle, AssetRemovedEvent {
            asset_address: asset,
            balance: balance,
            receiver: asset_recipient
        });
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
    public entry fun set_contract_state(owner: &signer, state: bool) acquires GlobalStorage, RampEventStore {
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
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_addr).contract_state_changed_event_handle, ContractStateChangedEvent { new_state: state });
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
    public entry fun set_owner(owner: &signer, new_owner: address) acquires GlobalStorage, RampEventStore {
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

        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_address).owner_changed_event_handle, OwnerChangedEvent { new_owner });
    }


    /// Deposit a fungible asset into the asset's vault
    /// # Parameters
    ///  - `user`: The signer who is depositing the asset
    ///  - `asset`: The asset to be deposited
    ///  - `amount`: The amount of the asset to be deposited
    /// # Notice
    ///  - The asset must be listed in the vault_address simple_map
    ///  - Emits a RampDeposit event when the asset is deposited
    ///  - This function acquires the GlobalStorage resource
    public entry fun on_ramp_deposit(user: &signer, asset: Object<Metadata>, amount: u64) acquires GlobalStorage, RampEventStore {
        let obj_address = get_obj_address();

        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_CONTRACT_STATE));

        // get global storage reference
        let global_storage_ref = borrow_global_mut<GlobalStorage>(obj_address);

        // assert the asset is listed
        assert!(
            simple_map::contains_key(&global_storage_ref.vault_address, &asset),
            error::invalid_state(EASSET_NOT_ALLOWED)
        );
        let user_store = ensure_primary_store_exists(signer::address_of(user), asset);

        let fungible_asset = fungible_asset::withdraw(
            user,
            user_store,
            amount
        );

        // get the asset vault
        let asset_vault = simple_map::borrow_mut(&mut global_storage_ref.vault_address, &asset);

        let amount = fungible_asset::amount(&fungible_asset);
        // deposit the fa to the store
        fungible_asset::deposit(asset_vault.store, fungible_asset);

        let fee = asset_vault.asset_fee_percentage;
        let fee_amount = amount * fee / 100;

        asset_vault.asset_revenue = asset_vault.asset_revenue + fee_amount;
        // emit the depist event
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_address).deposit_event_handle, RampDeposit {
            asset: asset,
            amount: amount - fee_amount,
            sender: signer::address_of(user)
        });
    }

    /// Withdraw an asset for the vault
    /// # Parameters
    ///  - `admin`: The signer who is withdrawing the asset
    ///  - `asset`: The asset to be withdrawn
    ///  - `recipient`: The address of the recipient of the asset
    ///  - `amount`: The amount of the asset to be withdrawn
    /// # Notice
    ///  - The asset must be listed in the vault_address simple_map
    ///  - Emits a RampWithdraw event when the asset is withdrawn
    ///  - This function acquires the GlobalStorage resource
    public entry fun off_ramp_withdraw(admin: &signer, asset: Object<Metadata>, recipient: address, amount: u64) acquires GlobalStorage, RampEventStore {
        
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
        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_address).withdraw_event_handle, RampWithdraw {
            asset: asset,
            amount: amount,
            recipient: recipient
        });
    }

    /// Change the fee percentage charged on an asset
    /// # Parameters
    ///  - `admin`: The signer who is changing the fee
    ///  - `asset`: The asset to be changed
    ///  - `new_fee`: The new fee percentage
    /// # Notice
    ///  - The asset must be listed in the vault_address simple_map
    ///  - Emits an AssetFeeChanged event when the fee is changed
    ///  - This function acquires the GlobalStorage resource
    public entry fun set_fee(admin: &signer, asset: Object<Metadata>, new_fee: u64) acquires GlobalStorage, RampEventStore {
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

        event::emit_event(&mut borrow_global_mut<RampEventStore>(obj_address).asset_fee_changed_event_handle, AssetFeeChanged {
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

    #[view]
    public fun get_fee(asset: Object<Metadata>): u64 acquires GlobalStorage {
        let obj_address = get_obj_address();
        // Ensure the global storage object exists
        assert!(exists<GlobalStorage>(obj_address), error::not_found(ENO_ASSET));
        // Check if the asset is in the allowed assets simple_map
        simple_map::borrow(&borrow_global<GlobalStorage>(obj_address).vault_address, &asset).asset_fee_percentage
    }


    #[test_only]
    use std::option;

    #[test_only]
    struct TestCoin {}

    #[test_only]
    struct TestInfo has drop {
        metadata: Object<Metadata>,
        mint_ref: aptos_framework::fungible_asset::MintRef,
        owner: signer,
        admin: signer,
        user_one: signer
    }

    #[test_only]
    fun create_coin_and_mint<CoinType>(creator: &signer, amount: u64): coin::Coin<CoinType> {
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<CoinType>(
            creator,
            std::string::utf8(b"Test"),
            std::string::utf8(b"Test"),
            8,
            true,
        );

        //aptos_framework::managed_coin::register<CoinType>(creator);
        //create_coin_conversion_map<CoinType>(creator);
        let coin = coin::mint<CoinType>(amount, &mint_cap);
        coin::destroy_burn_cap(burn_cap);
        coin::destroy_freeze_cap(freeze_cap);
        coin::destroy_mint_cap(mint_cap);
        //coin::deposit<CoinType>(signer::address_of(creator), coin);
        coin
    }

    #[test_only(owner = @RampAptos, admin = @0xface, user_one = @0xCafe)]
    public fun initialize_test(owner: signer, admin: address, user_one: address): TestInfo acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for initialize...");
        std::debug::print(&msg);
        aptos_framework::account::create_account_for_test(signer::address_of(&owner));

        let token_metadata = &object::create_named_object(&owner, b"test");
        aptos_framework::primary_fungible_store::create_primary_store_enabled_fungible_asset(
            token_metadata,
            option::none(),
            std::string::utf8(b"test"),
            std::string::utf8(b"test"),
            8,
            std::string::utf8(b""),
            std::string::utf8(b""),
        );
        let mint_ref = fungible_asset::generate_mint_ref(token_metadata);
        let fa = fungible_asset::mint(&mint_ref, 1000);
        let metadata = fungible_asset::metadata_from_asset(&fa);
        let admin_signer = aptos_framework::account::create_account_for_test(admin);
        let user_one_signer = aptos_framework::account::create_account_for_test(user_one);
        let admin_store = ensure_primary_store_exists(signer::address_of(&admin_signer), metadata);
        aptos_framework::fungible_asset::deposit(admin_store, fa);

        assert!(
            fungible_asset::balance(admin_store) == 1000,
            error::invalid_argument(1)
        );
        init_module(&owner);
        set_owner(&owner, admin);

        TestInfo {
            metadata: metadata,
            mint_ref: mint_ref,
            owner: owner,
            admin: admin_signer,
            user_one: user_one_signer
        }
    }

    #[test(owner=@RampAptos)]
    fun test_init(owner: signer) {
        init_module(&owner);
    }

    #[test(owner = @RampAptos, admin= @0xface, user_one = @0xCafe)]
    fun test_add_asset(owner: signer, admin: address, user_one: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for add_asset...");
        std::debug::print(&msg);
        let test_info = initialize_test(owner, admin, user_one);
        let initial_amount = 100u64;
        let fa = fungible_asset::mint(&test_info.mint_ref, 1000);
        let user_fa = fungible_asset::mint(&test_info.mint_ref, 1000);
        let admin_store = ensure_primary_store_exists(signer::address_of(&test_info.admin), test_info.metadata);
        let user_store = ensure_primary_store_exists(signer::address_of(&test_info.user_one), test_info.metadata);
        aptos_framework::fungible_asset::deposit(admin_store, fa);
        aptos_framework::fungible_asset::deposit(user_store, user_fa);
        assert!(
            fungible_asset::balance(admin_store) >= initial_amount,
            error::invalid_argument(1)
        );
        add_asset(
            &test_info.admin, test_info.metadata, 1u64, initial_amount
        );

        assert!(event::was_event_emitted(
            &AssetAddedEvent {
                asset_address: test_info.metadata,
                fee_percentage: 1u64,
                initial_amount: initial_amount
            }
        ), 4);
        assert!(is_asset_allowed(test_info.metadata), 1);
    }

    #[test(owner = @RampAptos, admin= @0xface, user_one = @0xCafe)]
    fun test_remove_asset(owner: signer, admin: address, user_one: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for remove_asset...");
        std::debug::print(&msg);

        let test_info = initialize_test(owner, admin, user_one);
        let initial_amount = 100u64;
        let fa = fungible_asset::mint(&test_info.mint_ref, 1000);
        let user_fa = fungible_asset::mint(&test_info.mint_ref, 1000);
        let admin_store = ensure_primary_store_exists(signer::address_of(&test_info.admin), test_info.metadata);
        let user_store = ensure_primary_store_exists(signer::address_of(&test_info.user_one), test_info.metadata);
        aptos_framework::fungible_asset::deposit(admin_store, fa);
        aptos_framework::fungible_asset::deposit(user_store, user_fa);
        assert!(
            fungible_asset::balance(admin_store) >= initial_amount,
            error::invalid_argument(1)
        );
        add_asset(&test_info.admin, test_info.metadata, 1u64, initial_amount);

        remove_asset(&test_info.admin, test_info.metadata, signer::address_of(&test_info.user_one));

        assert!(event::was_event_emitted(
            &AssetRemovedEvent {
                asset_address: test_info.metadata,
                balance: initial_amount,
                receiver: signer::address_of(&test_info.user_one)
            }
        ), 4);
        assert!(!is_asset_allowed(test_info.metadata), 5);
    }

    #[test(owner = @RampAptos, admin= @0x2)]
    fun test_set_contract_state(owner: signer, admin: signer) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for set_contract_state...");
        std::debug::print(&msg);
        aptos_framework::account::create_account_for_test(signer::address_of(&owner));
        let test_info = initialize_test(owner, signer::address_of(&admin), signer::address_of(&admin));
        set_contract_state(&test_info.admin, false);
        assert!(is_active() == false, 1);
        assert!(event::was_event_emitted(
            &ContractStateChangedEvent { new_state: false }
        ), 2);
    }


    #[test(owner = @RampAptos, admin= @0x2, new_owner = @0x4)]
    fun test_set_owner(owner: signer, admin: signer, new_owner: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for set_owner...");
        std::debug::print(&msg);
        aptos_framework::account::create_account_for_test(signer::address_of(&owner));
        let test_info = initialize_test(owner, signer::address_of(&admin), new_owner);
        set_owner(&test_info.admin, new_owner);
        assert!(get_owner() == new_owner, 1);
        assert!(event::was_event_emitted(
            &OwnerChangedEvent { new_owner }
        ), 2);
    }

    #[test(owner = @RampAptos, admin= @0x2, user_1 = @0xCAFE)]
    fun test_set_fee(owner: signer, admin: address, user_1: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for set_fee...");
        std::debug::print(&msg);
        let test_info = initialize_test(owner, admin, user_1);
        let initial_amount = aptos_framework::primary_fungible_store::balance(admin, test_info.metadata);
        add_asset(&test_info.admin, test_info.metadata, 1u64, initial_amount);
        set_fee(&test_info.admin, test_info.metadata, 2u64);
        assert!(get_fee(test_info.metadata) == 2u64, 1);
        assert!(event::was_event_emitted(
            &AssetFeeChanged { asset: test_info.metadata, new_fee: 2u64 }
        ), 2);
    }

    #[test(owner = @RampAptos, admin= @0x2, user_1 = @0xCAFE)]
    fun test_get_fee(owner: signer, admin: address, user_1: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for get_fee...");
        std::debug::print(&msg);
        let test_info = initialize_test(owner, admin, user_1);
        let initial_amount = aptos_framework::primary_fungible_store::balance(admin, test_info.metadata);
        add_asset(&test_info.admin, test_info.metadata, 1u64, initial_amount);
        set_fee(&test_info.admin, test_info.metadata, 2u64);
        assert!(get_fee(test_info.metadata) == 2u64, 1);
    }

    #[test(owner = @RampAptos, admin= @0xface, user_one = @0xCafe)]
    fun test_on_ramp_deposit(owner: signer, admin: address, user_one: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for on_ramp_deposit...");
        std::debug::print(&msg);

        let test_info = initialize_test(owner, admin, user_one);
        let initial_amount = 1000u64;
        let fa = fungible_asset::mint(&test_info.mint_ref, initial_amount);
        let user_fa = fungible_asset::mint(&test_info.mint_ref, initial_amount);
        let admin_store = ensure_primary_store_exists(signer::address_of(&test_info.admin), test_info.metadata);
        let user_store = ensure_primary_store_exists(signer::address_of(&test_info.user_one), test_info.metadata);
        aptos_framework::fungible_asset::deposit(admin_store, fa);
        aptos_framework::fungible_asset::deposit(user_store, user_fa);
        assert!(
            fungible_asset::balance(admin_store) >= initial_amount,
            error::invalid_argument(1)
        );
        add_asset(&test_info.admin, test_info.metadata, 1u64, initial_amount);

        on_ramp_deposit(&test_info.user_one, test_info.metadata, 100u64);

        let fee = get_fee(test_info.metadata);
        let fee_amount = 100u64 * fee / 100;
        assert!(event::was_event_emitted(
            &RampDeposit {
                asset: test_info.metadata,
                amount: 100u64 - fee_amount,
                sender: signer::address_of(&test_info.user_one)
            }
        ), 4);
    }


    #[test(owner = @RampAptos, admin= @0xface, user_one = @0xCafe)]
    fun test_off_ramp_withdraw(owner: signer, admin: address, user_one: address) acquires GlobalStorage {
        let msg: std::string::String = std::string::utf8(b"Running test for off_ramp_withdraw...");
        std::debug::print(&msg);

        let test_info = initialize_test(owner, admin, user_one);
        let initial_amount = 1000u64;
        let fa = fungible_asset::mint(&test_info.mint_ref, initial_amount);
        let user_fa = fungible_asset::mint(&test_info.mint_ref, initial_amount);
        let admin_store = ensure_primary_store_exists(signer::address_of(&test_info.admin), test_info.metadata);
        let user_store = ensure_primary_store_exists(signer::address_of(&test_info.user_one), test_info.metadata);
        aptos_framework::fungible_asset::deposit(admin_store, fa);
        aptos_framework::fungible_asset::deposit(user_store, user_fa);
        assert!(
            fungible_asset::balance(admin_store) >= initial_amount,
            error::invalid_argument(1)
        );
        add_asset(&test_info.admin, test_info.metadata, 1u64, initial_amount);

        off_ramp_withdraw(&test_info.admin, test_info.metadata, signer::address_of(&test_info.user_one), 100u64);

        assert!(event::was_event_emitted(
            &RampWithdraw {
                asset: test_info.metadata,
                amount: 100u64,
                recipient: signer::address_of(&test_info.user_one)
            }
        ), 4);
    }

    ////#[test(owner = @RampAptos, admin= @0xface, user_one = @0xCafe)]
    ////fun test_add_coin(owner: signer, admin: address, user_one: address) {//acquires GlobalStorage {
    ////    let msg: std::string::String = std::string::utf8(b"Running test for add_coin...");
    ////    std::debug::print(&msg);
////
    ////    let coin = create_coin_and_mint<TestCoin>(&owner, 10);
////
    ////    let test_info = initialize_test(owner, admin, user_one);
////
    //    //let coin_amount = coin::value(&coin);
    //    //coin::deposit(signer::address_of(&test_info.admin), coin);
    //    //add_asset(&test_info.admin, test_info.metadata, 1u64, coin_amount);
////
    //    //assert!(event::was_event_emitted(
    //    //    &AssetAddedEvent {
    //    //        asset_address: test_info.metadata,
    //    //        fee_percentage: 1u64,
    //    //        initial_amount: coin_amount
    //    //    }
    //    //), 4);
    ////}
}