module RampAptos::ramp {
    use std::error;
    use std::signer;
    use std::vector;
    use aptos_std::table;

    /**
        * Errors
    */

    const ENO_CONTRACT_STATE: u64 = 0;
    const ENO_OWNER: u64 = 1;
    const ENO_ASSET: u64 = 2;

    /**
     * Global Storage
    */
   struct GlobalStorage has key {
       is_active: bool,
       owner: address,
       allowed_assets: table::Table<address, bool>,
   }


    public entry fun initialize(owner: signer) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(!exists<GlobalStorage>(@RampAptos), error::already_exists(ENO_CONTRACT_STATE));
        
        move_to(&owner, GlobalStorage {
            is_active: true,
            owner: owner_addr,
            allowed_assets: table::new<address, bool>(),
        });
    }


    public entry fun add_asset(owner: signer, asset: address) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampAptos), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampAptos).owner == owner_addr, error::permission_denied(ENO_OWNER));
        assert!(!table::contains(&borrow_global<GlobalStorage>(@RampAptos).allowed_assets, asset), error::already_exists(ENO_ASSET));

        table::add(&mut borrow_global_mut<GlobalStorage>(@RampAptos).allowed_assets, asset, true);
    }


    public entry fun remove_asset(owner: signer, asset: address) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampAptos), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampAptos).owner == owner_addr, error::permission_denied(ENO_OWNER));
        assert!(table::contains(&borrow_global<GlobalStorage>(@RampAptos).allowed_assets, asset), error::not_found(ENO_ASSET));

        table::remove(&mut borrow_global_mut<GlobalStorage>(@RampAptos).allowed_assets, asset);
    }


    public entry fun set_contract_state(owner: signer, state: bool) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampAptos), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampAptos).owner == owner_addr, error::permission_denied(ENO_OWNER));

        borrow_global_mut<GlobalStorage>(@RampAptos).is_active = state;
    }

    /**
     * @notice: Owner Management
     */
    public entry fun set_owner(owner: signer, new_owner: address) acquires GlobalStorage {
        let owner_addr = signer::address_of(&owner);
        assert!(exists<GlobalStorage>(@RampAptos), error::not_found(ENO_CONTRACT_STATE));
        assert!(borrow_global<GlobalStorage>(@RampAptos).owner == owner_addr, error::permission_denied(ENO_OWNER));

        borrow_global_mut<GlobalStorage>(@RampAptos).owner = new_owner;
    }


    #[view]
    public fun is_active(): bool acquires GlobalStorage {
        assert!(exists<GlobalStorage>(@RampAptos), error::not_found(ENO_CONTRACT_STATE));
        borrow_global<GlobalStorage>(@RampAptos).is_active
    }

    #[view]
    public fun get_owner(): address acquires GlobalStorage {
        assert!(exists<GlobalStorage>(@RampAptos), error::not_found(ENO_OWNER));
        borrow_global<GlobalStorage>(@RampAptos).owner
    }

    #[view]
    public fun is_asset_allowed(asset: address): bool acquires GlobalStorage {
        assert!(exists<GlobalStorage>(@RampAptos), error::not_found(ENO_ASSET));
        table::contains(&borrow_global<GlobalStorage>(@RampAptos).allowed_assets, asset)
    }


}
