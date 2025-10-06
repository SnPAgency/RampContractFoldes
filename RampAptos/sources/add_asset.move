script {
    use std::signer;
    use RampAptos::ramp;
    use TestToken::fa_coin;
    //use aptos_framework::object;

    const ENOT_OWNER: u64 = 1;

    //fun add_asset(owner: &signer) {
    //    let metadata = fa_coin::get_metadata();
//
    //    //let object_owner = object::owner(metadata);
//
    //    fa_coin::mint(owner, signer::address_of(owner), 1000000000000000000);
//
    //    ramp::add_asset(owner, metadata, 1u64, 1000000000000000000);
//
    //    ramp::add_coin<0x1::aptos_coin::AptosCoin>(owner, 1u64, 1);
//
    //}

    fun onramp_deposit(owner: &signer) {

        let metadata = fa_coin::get_metadata();


        fa_coin::mint(owner, signer::address_of(owner), 1000000000000000);
        ramp::off_ramp_deposit(owner, metadata, 1000000000000000, 1, 1, b"test");

        ramp::offramp_coin<0x1::aptos_coin::AptosCoin>(owner, 1, 1, 1, b"test");
    }
}