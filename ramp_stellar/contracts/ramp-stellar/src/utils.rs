    use soroban_sdk::{Address, Bytes, Env};
    
    pub fn zero_address(env: &Env) -> Address {
        Address::from_string_bytes(&Bytes::from_array(env, &[0u8; 32]))
    }