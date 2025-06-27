use crate::{
    central_state,
    constants::{
        REFERRER_DISCOUNT_AND_FEE, ROOT_DOMAIN_ACCOUNT, VAULT_OWNER, VAULT_OWNER_DEPRECATED,
    },
    processor::create_split_v2,
};
use bonfida_utils::{
    checks::{check_account_key, check_account_owner},
    fp_math::fp32_div,
    tokens::SupportedToken,
};

use solana_program::{
    account_info::AccountInfo, clock::Clock, hash::hashv, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar,
};

use spl_name_service::state::{get_seeds_and_key, HASH_PREFIX};
use spl_token::state::Account;
use unicode_segmentation::UnicodeSegmentation;

pub fn get_usd_price(len: usize) -> u64 {
    let multiplier = match len {
        1 => 750,
        2 => 700,
        3 => 640,
        4 => 160,
        i if i < 10 => 20,
        _ => 1,
    };
    #[cfg(not(feature = "devnet"))]
    return multiplier * 1_000_000;
    #[cfg(feature = "devnet")]
    return multiplier * 1_000;
}

pub fn get_grapheme_len(name: &str) -> usize {
    name.graphemes(true).count()
}

pub fn get_hashed_name(name: &str) -> Vec<u8> {
    hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec()
}

pub fn get_name_key(name: &str, parent: Option<&Pubkey>) -> Result<Pubkey, ProgramError> {
    let hashed_name = get_hashed_name(name);
    let (name_account_key, _) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed_name,
        None,
        parent.map_or(Some(&ROOT_DOMAIN_ACCOUNT), Some),
    );
    Ok(name_account_key)
}

pub fn get_reverse_key(
    domain_key: &Pubkey,
    parent_key: Option<&Pubkey>,
) -> Result<Pubkey, ProgramError> {
    let hashed_reverse_lookup = get_hashed_name(&domain_key.to_string());
    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_reverse_lookup,
        Some(&central_state::KEY),
        parent_key,
    );
    Ok(reverse_lookup_account_key)
}

pub fn get_special_discount_and_fee(referrer_key: &Pubkey) -> (Option<u8>, Option<u8>) {
    #[cfg(feature = "no-special-discount-fee")]
    {
        (Some(13), Some(7))
    }
    #[cfg(not(feature = "no-special-discount-fee"))]
    {
        let now = Clock::get().unwrap().unix_timestamp as u64;
        let mut discount = None;
        let mut fee = None;

        let value = REFERRER_DISCOUNT_AND_FEE.get(&referrer_key.to_string());
        if let Some((
            discount_pct,
            start_time_d,
            end_time_d,
            referrer_pct,
            start_time_p,
            end_time_p,
        )) = value
        {
            discount = if *start_time_d < now && now < *end_time_d && discount_pct.is_some() {
                *discount_pct
            } else {
                None
            };

            fee = if *start_time_p < now && now < *end_time_p && referrer_pct.is_some() {
                *referrer_pct
            } else {
                None
            };
        }
        (discount, fee)
    }
}

pub struct PythAccounts<'a, 'b> {
    pub pyth_mapping_acc_or_feed: &'a AccountInfo<'b>,
    pub buyer_token_mint: Pubkey,
}

impl<'a, 'b: 'a> From<&'_ create_split_v2::Accounts<'a, AccountInfo<'b>>> for PythAccounts<'a, 'b> {
    fn from(value: &create_split_v2::Accounts<'a, AccountInfo<'b>>) -> Self {
        let buyer_token_mint =
            spl_token::state::Account::unpack_from_slice(&value.buyer_token_source.data.borrow())
                .unwrap()
                .mint;
        Self {
            pyth_mapping_acc_or_feed: value.pyth_feed_account,
            buyer_token_mint,
        }
    }
}

pub fn get_domain_price_checked<'a, 'b: 'a>(
    domain_name: &str,
    accounts: &create_split_v2::Accounts<'a, AccountInfo<'b>>,
) -> Result<u64, ProgramError> {
    let usd_price = get_usd_price(get_grapheme_len(domain_name));
    msg!("Registering domain for {}", usd_price);
    let buyer_token_mint =
        spl_token::state::Account::unpack_from_slice(&accounts.buyer_token_source.data.borrow())
            .unwrap()
            .mint;

    let token_price =
        get_token_usd_price_checked_v2(accounts.pyth_feed_account, &buyer_token_mint)?;
    let domain_price = fp32_div(usd_price, token_price).unwrap();

    Ok(domain_price)
}

pub fn get_token_usd_price_checked_v2(
    pyth_feed: &AccountInfo<'_>,
    mint: &Pubkey,
) -> Result<u64, ProgramError> {
    let token = SupportedToken::from_mint(mint)?;
    check_account_key(pyth_feed, &token.price_feed_account_key())?;
    let token_price = bonfida_utils::pyth::get_oracle_price_fp32_v2(
        mint,
        pyth_feed,
        token.decimals(),
        6,
        &Clock::get().unwrap(),
        60,
    )?;
    Ok(token_price)
}

pub fn check_vault_token_account_owner(account: &AccountInfo) -> Result<Account, ProgramError> {
    check_account_owner(account, &spl_token::ID)?;
    let token_account = Account::unpack_from_slice(&account.data.borrow())?;

    if token_account.owner != VAULT_OWNER && token_account.owner != VAULT_OWNER_DEPRECATED {
        return Err(ProgramError::IllegalOwner);
    }

    Ok(token_account)
}

#[test]
pub fn test_length() {
    let string_1 = "1".to_string();
    let string_2 = "12".to_string();
    let string_3 = "jkfdgnjkdfgn".to_string();
    let string_4 = "😀".to_string();
    let string_5 = "◎x".to_string();
    let string_6 = "🏳️‍🌈".to_string();

    assert_eq!(get_grapheme_len(&string_1), 1);
    assert_eq!(get_grapheme_len(&string_2), 2);
    assert_eq!(get_grapheme_len(&string_3), 12);
    assert_eq!(get_grapheme_len(&string_4), 1);
    assert_eq!(get_grapheme_len(&string_5), 2);
    assert_eq!(get_grapheme_len(&string_6), 1);
}
