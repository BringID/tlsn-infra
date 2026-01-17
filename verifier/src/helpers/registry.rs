use std::str::FromStr;
use alloy::hex::FromHexError;
use alloy::primitives::Address;

pub fn registry_from_string(
    registry_address: String
) -> Result<Address, FromHexError> {
    Address::from_str(registry_address.as_str())
}