// Find all our documentation at https://docs.near.org

pub mod events;
pub mod ft_core;
pub mod internal;
pub mod metadata;
pub mod storage;

use near_sdk::borsh::BorshDeserialize;
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::LazyOption;
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::AccountId;
use near_sdk::BorshStorageKey;
use near_sdk::NearToken;
use near_sdk::PanicOnDefault;
use near_sdk::StorageUsage;

use crate::events::*;
use crate::metadata::*;

const DATA_IMAGE_SVG_GT_ICON: &str = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDI0LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPgo8c3ZnIHZlcnNpb249IjEuMSIgaWQ9IkxheWVyXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHg9IjBweCIgeT0iMHB4IgoJIHZpZXdCb3g9IjAgMCA5MC4xIDkwIiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCA5MC4xIDkwOyIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+CjxwYXRoIGQ9Ik03Mi4yLDQuNkw1My40LDMyLjVjLTEuMywxLjksMS4yLDQuMiwzLDIuNkw3NC45LDE5YzAuNS0wLjQsMS4yLTAuMSwxLjIsMC42djUwLjNjMCwwLjctMC45LDEtMS4zLDAuNWwtNTYtNjcKCUMxNywxLjIsMTQuNCwwLDExLjUsMGgtMkM0LjMsMCwwLDQuMywwLDkuNnY3MC44QzAsODUuNyw0LjMsOTAsOS42LDkwYzMuMywwLDYuNC0xLjcsOC4yLTQuNmwxOC44LTI3LjljMS4zLTEuOS0xLjItNC4yLTMtMi42CglsLTE4LjUsMTZjLTAuNSwwLjQtMS4yLDAuMS0xLjItMC42VjIwLjFjMC0wLjcsMC45LTEsMS4zLTAuNWw1Niw2N2MxLjgsMi4yLDQuNSwzLjQsNy4zLDMuNGgyYzUuMywwLDkuNi00LjMsOS42LTkuNlY5LjYKCWMwLTUuMy00LjMtOS42LTkuNi05LjZDNzcuMSwwLDc0LDEuNyw3Mi4yLDQuNnoiLz4KPC9zdmc+"; // Base64 encoded SVG image

/// The specific version of the standard we're using
pub const FT_METADATA_SPEC: &str = "ft-1.0.0";

pub const ZERO_TOKEN: NearToken = NearToken::from_yoctonear(0);

// Implement the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    /// Keep track of each account's balances
    pub accounts: LookupMap<AccountId, NearToken>,

    /// Total supply of all tokens.
    pub total_supply: NearToken,

    /// The bytes for the largest possible account ID that can be registered on the contract
    pub bytes_for_longest_account_id: StorageUsage,

    /// Metadata for the contract itself
    pub metadata: LazyOption<FungibleTokenMetadata>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    Accounts,
    Metadata,
}
#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        // Calls the other function "new: with some default metadata and the owner_id & total supply passed in
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Team Token FT Tutorial".to_string(),
                symbol: "gtNEAR".to_string(),
                icon: Some(DATA_IMAGE_SVG_GT_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        let casted_total_supply = NearToken::from_yoctonear(total_supply.0);
        // Create a variable of type Self with all the fields initialized.
        let mut this = Self {
            // Set the total supply
            total_supply: casted_total_supply,
            // Set the bytes for the longest account ID to 0 temporarily until it's calculated later
            bytes_for_longest_account_id: 0,
            // Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            accounts: LookupMap::new(StorageKey::Accounts),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        };

        // Measure the bytes for the longest account ID and store it in the contract.
        this.measure_bytes_for_longest_account_id();

        // Register the owner's account and set their balance to the total supply.
        this.internal_register_account(&owner_id);
        this.internal_deposit(&owner_id, casted_total_supply);

        // Emit an event showing that the FTs were minted
        FtMint {
            owner_id: &owner_id,
            amount: &casted_total_supply,
            memo: Some("Initial token supply is minted"),
        }
        .emit();

        // Return the Contract object
        this
    }
}
