// Adapted from the mpl_token_metadata crate

use std::convert::TryFrom;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
    TokenOwnedEscrow,
    TokenRecord,
    MetadataDelegate,
    EditionMarkerV2,
    HolderDelegate,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum TokenStandard {
    NonFungible,
    FungibleAsset,
    Fungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
    ProgrammableNonFungibleEdition,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Metadata {
    pub key: Key,
    pub update_authority: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
    pub edition_nonce: Option<u8>,
    pub token_standard: Option<TokenStandard>,
    pub collection: Option<Collection>,
    pub uses: Option<Uses>,
    pub collection_details: Option<CollectionDetails>,
    pub programmable_config: Option<ProgrammableConfig>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum CollectionDetails {
    V1 { size: u64 },
    V2 { padding: [u8; 8] },
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum ProgrammableConfig {
    V1 { rule_set: Option<Pubkey> },
}

impl Metadata {
    /// Prefix values used to generate a PDA for this account.
    ///
    /// Values are positional and appear in the following order:
    ///
    ///   0. `Metadata::PREFIX`
    ///   1. `crate::MPL_TOKEN_METADATA_ID`
    ///   2. mint (`Pubkey`)
    pub const PREFIX: &'static [u8] = "metadata".as_bytes();

    pub fn create_pda(
        mint: Pubkey,
        bump: u8,
    ) -> Result<solana_program::pubkey::Pubkey, solana_program::pubkey::PubkeyError> {
        solana_program::pubkey::Pubkey::create_program_address(
            &[
                "metadata".as_bytes(),
                crate::constants::MPL_TOKEN_METADATA_PROGRAM.as_ref(),
                mint.as_ref(),
                &[bump],
            ],
            &crate::constants::MPL_TOKEN_METADATA_PROGRAM,
        )
    }

    pub fn find_pda(mint: &Pubkey) -> (solana_program::pubkey::Pubkey, u8) {
        solana_program::pubkey::Pubkey::find_program_address(
            &[
                "metadata".as_bytes(),
                crate::constants::MPL_TOKEN_METADATA_PROGRAM.as_ref(),
                mint.as_ref(),
            ],
            &crate::constants::MPL_TOKEN_METADATA_PROGRAM,
        )
    }
}

impl<'a> TryFrom<&solana_program::account_info::AccountInfo<'a>> for Metadata {
    type Error = std::io::Error;

    fn try_from(
        account_info: &solana_program::account_info::AccountInfo<'a>,
    ) -> Result<Self, Self::Error> {
        let mut data: &[u8] = &(*account_info.data).borrow();
        Self::deserialize(&mut data)
    }
}

#[derive(BorshSerialize)]
pub struct MasterEdition {
    pub key: Key,
    pub supply: u64,
    pub max_supply: Option<u64>,
}

impl MasterEdition {
    /// Prefix values used to generate a PDA for this account.
    ///
    /// Values are positional and appear in the following order:
    ///
    ///   0. `MasterEdition::PREFIX.0`
    ///   1. `crate::MPL_TOKEN_METADATA_ID`
    ///   2. mint (`Pubkey`)
    ///   3. `MasterEdition::PREFIX.1`
    pub const PREFIX: (&'static [u8], &'static [u8]) =
        ("metadata".as_bytes(), "edition".as_bytes());

    pub fn create_pda(
        mint: Pubkey,
        bump: u8,
    ) -> Result<solana_program::pubkey::Pubkey, solana_program::pubkey::PubkeyError> {
        solana_program::pubkey::Pubkey::create_program_address(
            &[
                "metadata".as_bytes(),
                crate::constants::MPL_TOKEN_METADATA_PROGRAM.as_ref(),
                mint.as_ref(),
                "edition".as_bytes(),
                &[bump],
            ],
            &crate::constants::MPL_TOKEN_METADATA_PROGRAM,
        )
    }

    pub fn find_pda(mint: &Pubkey) -> (solana_program::pubkey::Pubkey, u8) {
        solana_program::pubkey::Pubkey::find_program_address(
            &[
                "metadata".as_bytes(),
                crate::constants::MPL_TOKEN_METADATA_PROGRAM.as_ref(),
                mint.as_ref(),
                "edition".as_bytes(),
            ],
            &crate::constants::MPL_TOKEN_METADATA_PROGRAM,
        )
    }
}

/// `burn_nft` CPI instruction.
pub struct BurnNftCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Metadata (pda of ['metadata', program id, mint id])
    pub metadata: &'b solana_program::account_info::AccountInfo<'a>,
    /// NFT owner
    pub owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// Mint of the NFT
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account to close
    pub token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// MasterEdition2 of the NFT
    pub master_edition_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// SPL Token Program
    pub spl_token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Metadata of the Collection
    pub collection_metadata: Option<&'b solana_program::account_info::AccountInfo<'a>>,
}

pub struct BurnNftCpiAccounts<'a, 'b> {
    /// Metadata (pda of ['metadata', program id, mint id])
    pub metadata: &'b solana_program::account_info::AccountInfo<'a>,
    /// NFT owner
    pub owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// Mint of the NFT
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account to close
    pub token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// MasterEdition2 of the NFT
    pub master_edition_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// SPL Token Program
    pub spl_token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Metadata of the Collection
    pub collection_metadata: Option<&'b solana_program::account_info::AccountInfo<'a>>,
}

impl<'a, 'b> BurnNftCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: BurnNftCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            metadata: accounts.metadata,
            owner: accounts.owner,
            mint: accounts.mint,
            token_account: accounts.token_account,
            master_edition_account: accounts.master_edition_account,
            spl_token_program: accounts.spl_token_program,
            collection_metadata: accounts.collection_metadata,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(7 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.metadata.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.owner.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.master_edition_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.spl_token_program.key,
            false,
        ));
        if let Some(collection_metadata) = self.collection_metadata {
            accounts.push(solana_program::instruction::AccountMeta::new(
                *collection_metadata.key,
                false,
            ));
        }
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = borsh::to_vec(&BurnNftInstructionData::new()).unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::constants::MPL_TOKEN_METADATA_PROGRAM,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(7 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.metadata.clone());
        account_infos.push(self.owner.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.token_account.clone());
        account_infos.push(self.master_edition_account.clone());
        account_infos.push(self.spl_token_program.clone());
        if let Some(collection_metadata) = self.collection_metadata {
            account_infos.push(collection_metadata.clone());
        }
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
struct BurnNftInstructionData {
    discriminator: u8,
}

impl BurnNftInstructionData {
    fn new() -> Self {
        Self { discriminator: 29 }
    }
}
