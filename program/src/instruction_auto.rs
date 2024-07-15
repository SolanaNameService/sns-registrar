use crate::processor::{create, create_reverse, create_split_v2, create_with_nft, delete};
use bonfida_utils::InstructionsAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Create a reverse lookup registry for a name registry
    ///
    /// | Index | Writable | Signer | Description                                     |
    /// | --------------------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The name service program account                |
    /// | 1     | ❌        | ❌      | The root domain account                         |
    /// | 2     | ✅        | ❌      | The reverse lookup account                      |
    /// | 3     | ❌        | ❌      | The system program account                      |
    /// | 4     | ❌        | ❌      | The central state account                       |
    /// | 5     | ✅        | ✅      | The fee payer account                           |
    /// | 6     | ❌        | ❌      |                                                 |
    /// | 7     | ✅        | ❌      | The optional parent name account for subdomains |
    /// | 8     | ✅        | ✅      | The optional parent name owner                  |
    CreateReverse = 12,
    /// Create a domain name and buy the ownership of a domain name
    ///
    /// | Index | Writable | Signer | Description                                                         |
    /// | ----------------------------------------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The naming service program ID                                       |
    /// | 1     | ❌        | ❌      | The root domain account                                             |
    /// | 2     | ✅        | ❌      | The name account                                                    |
    /// | 3     | ✅        | ❌      | The reverse look up account                                         |
    /// | 4     | ❌        | ❌      | The system program account                                          |
    /// | 5     | ❌        | ❌      | The central state account                                           |
    /// | 6     | ✅        | ✅      | The buyer account                                                   |
    /// | 7     | ✅        | ❌      | The buyer token account                                             |
    /// | 8     | ❌        | ❌      | The Pyth mapping account                                            |
    /// | 9     | ❌        | ❌      | The Pyth product account                                            |
    /// | 10    | ❌        | ❌      | The Pyth price account                                              |
    /// | 11    | ✅        | ❌      | The vault account                                                   |
    /// | 12    | ❌        | ❌      | The SPL token program                                               |
    /// | 13    | ❌        | ❌      | The rent sysvar account                                             |
    /// | 14    | ❌        | ❌      | The state auction account                                           |
    /// | 15    | ✅        | ❌      | The *optional* referrer token account to receive a portion of fees. |
    Create,
    /// Deprecated instruction
    _Claim,
    /// Deprecated instruction
    _EndAuction,
    /// Delete a domain and clean up related accounts
    ///
    /// | Index | Writable | Signer | Description |
    /// | --------------------------------------- |
    /// | 0     | ❌        | ❌      |             |
    /// | 1     | ❌        | ❌      |             |
    /// | 2     | ✅        | ❌      |             |
    /// | 3     | ✅        | ❌      |             |
    /// | 4     | ✅        | ❌      |             |
    /// | 5     | ✅        | ❌      |             |
    /// | 6     | ❌        | ❌      |             |
    /// | 7     | ❌        | ✅      |             |
    /// | 8     | ✅        | ❌      |             |
    Delete,
    /// Create a domain name with a wolf
    ///
    /// | Index | Writable | Signer | Description                             |
    /// | ------------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The naming service program ID           |
    /// | 1     | ❌        | ❌      | The root domain account                 |
    /// | 2     | ✅        | ❌      | The name account                        |
    /// | 3     | ✅        | ❌      | The reverse look up account             |
    /// | 4     | ❌        | ❌      | The system program account              |
    /// | 5     | ❌        | ❌      | The central state account               |
    /// | 6     | ✅        | ✅      | The buyer account                       |
    /// | 7     | ✅        | ❌      | The buyer token account                 |
    /// | 8     | ✅        | ❌      | The NFT metadata account                |
    /// | 9     | ✅        | ❌      | The NFT mint account                    |
    /// | 10    | ✅        | ❌      | The NFT master edition account          |
    /// | 11    | ✅        | ❌      | The NFT collection account              |
    /// | 12    | ❌        | ❌      | The SPL token program                   |
    /// | 13    | ❌        | ❌      | The rent sysvar account                 |
    /// | 14    | ❌        | ❌      | The state auction account               |
    /// | 15    | ❌        | ❌      | Metaplex token metadata program account |
    CreateWithNft,
    /// Deprecated instruction
    _CloseAuctionAccount,
    CreateSplit,
    CreateSplitV2,
}
#[allow(missing_docs)]
pub fn create(
    program_id: Pubkey,
    accounts: create::Accounts<Pubkey>,
    params: create::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::Create as u8, params)
}
#[allow(missing_docs)]
pub fn create_reverse(
    program_id: Pubkey,
    accounts: create_reverse::Accounts<Pubkey>,
    params: create_reverse::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CreateReverse as u8, params)
}

#[allow(missing_docs)]
pub fn delete(
    program_id: Pubkey,
    accounts: delete::Accounts<Pubkey>,
    params: delete::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::Delete as u8, params)
}
#[allow(missing_docs)]
pub fn create_with_nft(
    program_id: Pubkey,
    accounts: create_with_nft::Accounts<Pubkey>,
    params: create_with_nft::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CreateWithNft as u8, params)
}

#[allow(missing_docs)]
pub fn create_split_v2(
    program_id: Pubkey,
    accounts: create_split_v2::Accounts<Pubkey>,
    params: create_split_v2::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CreateSplitV2 as u8, params)
}
