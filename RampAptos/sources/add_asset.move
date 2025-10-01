script {
    use std::signer;
    use RampAptos::ramp;
    use RampAptos::fa_coin;
    //use aptos_framework::object;

    const ENOT_OWNER: u64 = 1;

    //fun add_asset(owner: &signer) {
    //    let metadata = fa_coin::get_metadata();
//
    //    //let object_owner = object::owner(metadata);
//
    //    fa_coin::mint(owner, signer::address_of(owner), 1000);
//
    //    ramp::add_asset(owner, metadata, 1u64, 1000);
    //}

    fun onramp_deposit(owner: &signer) {

        let metadata = fa_coin::get_metadata();

        fa_coin::mint(owner, signer::address_of(owner), 1000);

        ramp::on_ramp_deposit(owner, metadata, 1000, 1, 1, b"test");
    
    }
}