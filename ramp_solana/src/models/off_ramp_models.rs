use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum Region {
    KEN,
    NGA,
    UGA,
    RWA,
    GHN,
    EGY,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum Medium {
    Primary,
    Secondary,
    Tertiary,
}


