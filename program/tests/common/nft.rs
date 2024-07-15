use mpl_token_metadata::accounts::{MasterEdition, Metadata};
use mpl_token_metadata::types::{Collection, Key};
use solana_program::program_option::COption;
use {
    solana_program::{pubkey, pubkey::Pubkey},
    spl_token::state::Account,
};

// Example
//https://solana.fm/address/9AU6HXCuW4wtPFt8sudyYTgZ6QdnczTJBcpzvf2iLgZy?cluster=mainnet-qn1
pub const NFT_METADATA_KEY: Pubkey = pubkey!("6zvNxCM4WnMuiHKw1CLXbNGLq1JoszzGtNA2kZ5PezNv");
pub const COLLECTION_KEY: Pubkey = pubkey!("Dw74YSxTKVXsztPm3TmwbnfLK8KVaCZw69jVu4LE6uJe");
pub const NFT_MINT: Pubkey = pubkey!("9AU6HXCuW4wtPFt8sudyYTgZ6QdnczTJBcpzvf2iLgZy");
pub const UPDATE_AUTH: Pubkey = pubkey!("8bkVaWKWdr5ugXafNu74PDT6FoN494QbAuvYAEUNXxpY");
pub const MASTER_EDITION: Pubkey = pubkey!("DmWifqndenzMBfgUr4qhGsFgiFy2DhSKxowbNL9ko4FD");

pub fn get_metadata() -> Metadata {
    Metadata {
        key: Key::MetadataV1,
        update_authority: UPDATE_AUTH,
        mint: NFT_MINT,
        primary_sale_happened: true,
        is_mutable: true,
        edition_nonce: Some(255),
        token_standard: Some(mpl_token_metadata::types::TokenStandard::NonFungible),
        collection: Some(Collection {
            key: COLLECTION_KEY,
            verified: true,
        }),
        uses: None,
        collection_details: None,
        programmable_config: None,
        name: "".to_string(),
        uri: "".to_string(),
        seller_fee_basis_points: 0,
        symbol: "".to_string(),
        creators: None,
    }
}

pub fn get_nft_account(owner: &Pubkey) -> Account {
    Account {
        mint: NFT_MINT,
        owner: *owner,
        amount: 1,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
}

pub fn get_master_edition() -> MasterEdition {
    MasterEdition {
        key: Key::MasterEditionV2,
        supply: 0,
        max_supply: Some(0),
    }
}

pub fn get_collection() -> Metadata {
    Metadata {
        key: Key::MetadataV1,
        update_authority: UPDATE_AUTH,
        mint: COLLECTION_KEY,
        primary_sale_happened: false,
        is_mutable: true,
        edition_nonce: Some(255),
        token_standard: Some(mpl_token_metadata::types::TokenStandard::NonFungible),
        collection: None,
        uses: None,
        collection_details: None,
        programmable_config: None,
        name: "".to_owned(),
        symbol: "".to_owned(),
        uri: "".to_owned(),
        seller_fee_basis_points: 0,
        creators: None,
    }
}

#[cfg(test)]
mod test {
    use mpl_token_metadata::accounts::MasterEdition;

    use super::NFT_MINT;

    #[test]
    fn t() {
        let (k, _) = MasterEdition::find_pda(&NFT_MINT);
        println!("{k}")
    }
}
