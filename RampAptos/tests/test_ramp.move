#[test_only]
module RampAptos::ramp_test {
    use RampAptos::ramp;
    use std::signer;
    use std::string;
    use std::debug;
    use aptos_framework::account;

    #[test(owner = @RampAptos, admin = @0x2)]
    public entry fun test_initialize(owner: signer, admin: address) {
        let msg: string::String = string::utf8(b"Running test for initialize...");
        debug::print(&msg);
    
        // Create a test account
        account::create_account_for_test(signer::address_of(&owner));

        ramp::initialize_test(owner, admin);

        let (is_active, controller) = ramp::get_global_storage();

        assert!(is_active, 1);
        assert!(controller == @0x2, 2);
    }

/*     #[test(owner = @RampAptos, admin= @0x2, asset = @0xCAFE)]
    public entry fun test_add_asset(owner: signer, admin: signer, asset: address) {
        let msg: string::String = string::utf8(b"Running test for add_asset...");
        debug::print(&msg);

        aptos_framework::account::create_account_for_test(signer::address_of(&owner));
        ramp::initialize_test(owner, signer::address_of(&admin));
        ramp::add_asset(&admin, asset);

        assert!(event::was_event_emitted(
            &AssetAddedEvent { asset_address: asset }
        ), 4);
        assert!(ramp::is_asset_allowed(asset), 1);
    } */
}