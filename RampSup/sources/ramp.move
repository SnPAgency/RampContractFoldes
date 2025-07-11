module RampSup::ramp {
    use std::error;
    use std::signer;
    use std::vector;
    use aptos_std::table;
    use supra_framework::object;

    /**
        * Errors
    */

    const ENO_CONTRACT_STATE: u64 = 0;
    const ENO_OWNER: u64 = 1;
    const ENO_ASSET: u64 = 2;

    /**
     * Global Storage
    */

    // storage object name
    const RAMP_SUP: vector<u8> = b"SUPRA_RAMP_GLOBAL_STORAGE";


    #[resource_group_member(group = supra_framework::object::ObjectGroup)]
    struct GlobalStorage has key {
        is_active: bool,
        owner: address,
        allowed_assets: table::Table<address, bool>,
    }


    public entry fun initialize(owner: &signer) acquires GlobalStorage {
        // create a global storage object to house all global storage
        let constructor_ref = object::create_named_object(owner, RAMP_SUP);

        // generate a signer for the global storage object
        let global_signer = &object::generate_signer(&constructor_ref);

        move_to(global_signer, GlobalStorage {
            is_active: true,
            owner: signer::address_of(owner),
            allowed_assets: table::new<address, bool>(),
        });
    }


    public entry fun add_asset(owner: signer, asset: address) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampSup), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampSup).owner == owner_addr, error::permission_denied(ENO_OWNER));
        assert!(!table::contains(&borrow_global<GlobalStorage>(@RampSup).allowed_assets, asset), error::already_exists(ENO_ASSET));

        table::add(&mut borrow_global_mut<GlobalStorage>(@RampSup).allowed_assets, asset, true);
    }


    public entry fun remove_asset(owner: signer, asset: address) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampSup), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampSup).owner == owner_addr, error::permission_denied(ENO_OWNER));
        assert!(table::contains(&borrow_global<GlobalStorage>(@RampSup).allowed_assets, asset), error::not_found(ENO_ASSET));

        table::remove(&mut borrow_global_mut<GlobalStorage>(@RampSup).allowed_assets, asset);
    }


    public entry fun set_contract_state(owner: signer, state: bool) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampSup), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampSup).owner == owner_addr, error::permission_denied(ENO_OWNER));

        borrow_global_mut<GlobalStorage>(@RampSup).is_active = state;
    }

    /**
     * @notice: Owner Management
     */
    public entry fun set_owner(owner: signer, new_owner: address) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampSup), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampSup).owner == owner_addr, error::permission_denied(ENO_OWNER));

        borrow_global_mut<GlobalStorage>(@RampSup).owner = new_owner;
    }


    #[view]
    public fun is_active(): bool acquires GlobalStorage {
        assert!(exists<GlobalStorage>(@RampSup), error::not_found(ENO_CONTRACT_STATE));
        borrow_global<GlobalStorage>(@RampSup).is_active
    }

    #[view]
    public fun get_owner(): address acquires GlobalStorage {
        assert!(exists<GlobalStorage>(@RampSup), error::not_found(ENO_OWNER));
        borrow_global<GlobalStorage>(@RampSup).owner
    }

    #[view]
    public fun is_asset_allowed(asset: address): bool acquires GlobalStorage {
        assert!(exists<GlobalStorage>(@RampSup), error::not_found(ENO_ASSET));
        table::contains(&borrow_global<GlobalStorage>(@RampSup).allowed_assets, asset)
    }


}
