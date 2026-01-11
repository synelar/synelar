use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Burn};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata,
    mpl_token_metadata::types::DataV2,
};

declare_id!("SYNiD1111111111111111111111111111111111111");

#[program]
pub mod synid {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, mint_price: u64, access_fee: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.mint_count = 0;
        config.mint_price = mint_price;
        config.access_fee = access_fee;
        config.treasury = ctx.accounts.treasury.key();
        config.paused = false;
        config.total_revenue = 0;
        config.bump = ctx.bumps.config;
        Ok(())
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        mint_price: Option<u64>,
        access_fee: Option<u64>,
        paused: Option<bool>,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        if let Some(price) = mint_price {
            config.mint_price = price;
        }
        if let Some(fee) = access_fee {
            config.access_fee = fee;
        }
        if let Some(p) = paused {
            config.paused = p;
        }
        Ok(())
    }

    pub fn mint_synid(
        ctx: Context<MintSynid>,
        name: String,
        uri: String,
        encrypted_cid: String,
        encryption_key_hash: [u8; 32],
    ) -> Result<()> {
        require!(!ctx.accounts.config.paused, SynidError::Paused);
        require!(name.len() <= 32, SynidError::NameTooLong);
        require!(uri.len() <= 200, SynidError::UriTooLong);
        require!(encrypted_cid.len() <= 128, SynidError::CidTooLong);

        let config = &mut ctx.accounts.config;
        
        if config.mint_price > 0 {
            let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.owner.key(),
                &config.treasury,
                config.mint_price,
            );
            anchor_lang::solana_program::program::invoke(
                &transfer_ix,
                &[
                    ctx.accounts.owner.to_account_info(),
                    ctx.accounts.treasury.to_account_info(),
                ],
            )?;
            config.total_revenue += config.mint_price;
        }

        config.mint_count += 1;

        let synid = &mut ctx.accounts.synid;
        synid.owner = ctx.accounts.owner.key();
        synid.mint = ctx.accounts.mint.key();
        synid.encrypted_cid = encrypted_cid;
        synid.encryption_key_hash = encryption_key_hash;
        synid.created_at = Clock::get()?.unix_timestamp;
        synid.updated_at = Clock::get()?.unix_timestamp;
        synid.token_id = config.mint_count;
        synid.soulbound = true;
        synid.access_count = 0;
        synid.total_earnings = 0;
        synid.reputation_score = 100;
        synid.verified = false;
        synid.bump = ctx.bumps.synid;

        let seeds = &[b"mint_authority".as_ref(), &[ctx.bumps.mint_authority]];
        let signer_seeds = &[&seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        let data = DataV2 {
            name,
            symbol: "SYNID".to_string(),
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    mint_authority: ctx.accounts.mint_authority.to_account_info(),
                    payer: ctx.accounts.owner.to_account_info(),
                    update_authority: ctx.accounts.mint_authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                signer_seeds,
            ),
            data,
            true,
            true,
            None,
        )?;

        emit!(SynidMinted {
            owner: ctx.accounts.owner.key(),
            mint: ctx.accounts.mint.key(),
            token_id: synid.token_id,
            timestamp: synid.created_at,
        });

        Ok(())
    }

    pub fn update_profile(
        ctx: Context<UpdateProfile>,
        encrypted_cid: Option<String>,
        encryption_key_hash: Option<[u8; 32]>,
    ) -> Result<()> {
        let synid = &mut ctx.accounts.synid;
        
        if let Some(cid) = encrypted_cid {
            require!(cid.len() <= 128, SynidError::CidTooLong);
            synid.encrypted_cid = cid;
        }
        if let Some(hash) = encryption_key_hash {
            synid.encryption_key_hash = hash;
        }
        synid.updated_at = Clock::get()?.unix_timestamp;

        emit!(ProfileUpdated {
            owner: ctx.accounts.owner.key(),
            timestamp: synid.updated_at,
        });

        Ok(())
    }

    pub fn request_access(
        ctx: Context<RequestAccess>,
        fields: Vec<String>,
        offered_payment: u64,
        expires_at: i64,
    ) -> Result<()> {
        require!(fields.len() <= 10, SynidError::TooManyFields);
        require!(offered_payment >= ctx.accounts.config.access_fee, SynidError::InsufficientPayment);

        let request = &mut ctx.accounts.access_request;
        request.synid = ctx.accounts.synid.key();
        request.requester = ctx.accounts.requester.key();
        request.fields = fields.clone();
        request.offered_payment = offered_payment;
        request.created_at = Clock::get()?.unix_timestamp;
        request.expires_at = expires_at;
        request.status = AccessStatus::Pending;
        request.bump = ctx.bumps.access_request;

        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.requester.key(),
            &ctx.accounts.escrow.key(),
            offered_payment,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.requester.to_account_info(),
                ctx.accounts.escrow.to_account_info(),
            ],
        )?;

        emit!(AccessRequested {
            synid: ctx.accounts.synid.key(),
            requester: ctx.accounts.requester.key(),
            fields,
            payment: offered_payment,
            timestamp: request.created_at,
        });

        Ok(())
    }

    pub fn approve_access(ctx: Context<ApproveAccess>) -> Result<()> {
        let request = &mut ctx.accounts.access_request;
        require!(request.status == AccessStatus::Pending, SynidError::InvalidStatus);
        require!(Clock::get()?.unix_timestamp < request.expires_at, SynidError::RequestExpired);

        request.status = AccessStatus::Approved;

        let synid = &mut ctx.accounts.synid;
        synid.access_count += 1;
        synid.total_earnings += request.offered_payment;

        let config = &ctx.accounts.config;
        let platform_fee = request.offered_payment * 5 / 100;
        let owner_payment = request.offered_payment - platform_fee;

        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= request.offered_payment;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += owner_payment;
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? += platform_fee;

        let grant = &mut ctx.accounts.access_grant;
        grant.synid = ctx.accounts.synid.key();
        grant.requester = request.requester;
        grant.fields = request.fields.clone();
        grant.payment = request.offered_payment;
        grant.granted_at = Clock::get()?.unix_timestamp;
        grant.expires_at = request.expires_at;
        grant.active = true;
        grant.bump = ctx.bumps.access_grant;

        emit!(AccessApproved {
            synid: ctx.accounts.synid.key(),
            requester: request.requester,
            payment: request.offered_payment,
            timestamp: grant.granted_at,
        });

        Ok(())
    }

    pub fn deny_access(ctx: Context<DenyAccess>) -> Result<()> {
        let request = &mut ctx.accounts.access_request;
        require!(request.status == AccessStatus::Pending, SynidError::InvalidStatus);

        request.status = AccessStatus::Denied;

        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= request.offered_payment;
        **ctx.accounts.requester.to_account_info().try_borrow_mut_lamports()? += request.offered_payment;

        emit!(AccessDenied {
            synid: ctx.accounts.synid.key(),
            requester: request.requester,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn revoke_access(ctx: Context<RevokeAccess>) -> Result<()> {
        let grant = &mut ctx.accounts.access_grant;
        require!(grant.active, SynidError::AlreadyRevoked);
        grant.active = false;

        emit!(AccessRevoked {
            synid: ctx.accounts.synid.key(),
            requester: grant.requester,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn verify_identity(ctx: Context<VerifyIdentity>) -> Result<()> {
        let synid = &mut ctx.accounts.synid;
        synid.verified = true;
        synid.reputation_score = synid.reputation_score.saturating_add(50);

        emit!(IdentityVerified {
            owner: synid.owner,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn update_reputation(ctx: Context<UpdateReputation>, delta: i16) -> Result<()> {
        let synid = &mut ctx.accounts.synid;
        let new_score = (synid.reputation_score as i32 + delta as i32).clamp(0, 1000) as u16;
        synid.reputation_score = new_score;

        emit!(ReputationUpdated {
            owner: synid.owner,
            new_score,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn burn_synid(ctx: Context<BurnSynid>) -> Result<()> {
        let synid = &ctx.accounts.synid;

        let seeds = &[b"mint_authority".as_ref(), &[ctx.bumps.mint_authority]];
        let signer_seeds = &[&seeds[..]];

        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        emit!(SynidBurned {
            owner: ctx.accounts.owner.key(),
            token_id: synid.token_id,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Config::SIZE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump, has_one = authority)]
    pub config: Account<'info, Config>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintSynid<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = owner,
        space = 8 + SynidAccount::SIZE,
        seeds = [b"synid", owner.key().as_ref()],
        bump
    )]
    pub synid: Account<'info, SynidAccount>,
    #[account(
        init,
        payer = owner,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: SystemAccount<'info>,
    #[account(mut)]
    pub metadata: SystemAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    #[account(mut, seeds = [b"synid", owner.key().as_ref()], bump = synid.bump, has_one = owner)]
    pub synid: Account<'info, SynidAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct RequestAccess<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub synid: Account<'info, SynidAccount>,
    #[account(
        init,
        payer = requester,
        space = 8 + AccessRequest::SIZE,
        seeds = [b"access_request", synid.key().as_ref(), requester.key().as_ref()],
        bump
    )]
    pub access_request: Account<'info, AccessRequest>,
    #[account(mut, seeds = [b"escrow"], bump)]
    pub escrow: SystemAccount<'info>,
    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveAccess<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut, has_one = owner)]
    pub synid: Account<'info, SynidAccount>,
    #[account(mut, has_one = synid)]
    pub access_request: Account<'info, AccessRequest>,
    #[account(
        init,
        payer = owner,
        space = 8 + AccessGrant::SIZE,
        seeds = [b"access_grant", synid.key().as_ref(), access_request.requester.as_ref()],
        bump
    )]
    pub access_grant: Account<'info, AccessGrant>,
    #[account(mut, seeds = [b"escrow"], bump)]
    pub escrow: SystemAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DenyAccess<'info> {
    #[account(has_one = owner)]
    pub synid: Account<'info, SynidAccount>,
    #[account(mut, has_one = synid)]
    pub access_request: Account<'info, AccessRequest>,
    #[account(mut, seeds = [b"escrow"], bump)]
    pub escrow: SystemAccount<'info>,
    #[account(mut)]
    pub requester: SystemAccount<'info>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevokeAccess<'info> {
    #[account(has_one = owner)]
    pub synid: Account<'info, SynidAccount>,
    #[account(mut)]
    pub access_grant: Account<'info, AccessGrant>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyIdentity<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = authority)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub synid: Account<'info, SynidAccount>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateReputation<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = authority)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub synid: Account<'info, SynidAccount>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct BurnSynid<'info> {
    #[account(mut, has_one = owner, has_one = mint)]
    pub synid: Account<'info, SynidAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: SystemAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = authority)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub mint_count: u64,
    pub mint_price: u64,
    pub access_fee: u64,
    pub treasury: Pubkey,
    pub paused: bool,
    pub total_revenue: u64,
    pub bump: u8,
}

impl Config {
    pub const SIZE: usize = 32 + 8 + 8 + 8 + 32 + 1 + 8 + 1;
}

#[account]
pub struct SynidAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub encrypted_cid: String,
    pub encryption_key_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
    pub token_id: u64,
    pub soulbound: bool,
    pub access_count: u64,
    pub total_earnings: u64,
    pub reputation_score: u16,
    pub verified: bool,
    pub bump: u8,
}

impl SynidAccount {
    pub const SIZE: usize = 32 + 32 + 132 + 32 + 8 + 8 + 8 + 1 + 8 + 8 + 2 + 1 + 1;
}

#[account]
pub struct AccessRequest {
    pub synid: Pubkey,
    pub requester: Pubkey,
    pub fields: Vec<String>,
    pub offered_payment: u64,
    pub created_at: i64,
    pub expires_at: i64,
    pub status: AccessStatus,
    pub bump: u8,
}

impl AccessRequest {
    pub const SIZE: usize = 32 + 32 + 260 + 8 + 8 + 8 + 2 + 1;
}

#[account]
pub struct AccessGrant {
    pub synid: Pubkey,
    pub requester: Pubkey,
    pub fields: Vec<String>,
    pub payment: u64,
    pub granted_at: i64,
    pub expires_at: i64,
    pub active: bool,
    pub bump: u8,
}

impl AccessGrant {
    pub const SIZE: usize = 32 + 32 + 260 + 8 + 8 + 8 + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AccessStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

#[event]
pub struct SynidMinted {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub token_id: u64,
    pub timestamp: i64,
}

#[event]
pub struct ProfileUpdated {
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AccessRequested {
    pub synid: Pubkey,
    pub requester: Pubkey,
    pub fields: Vec<String>,
    pub payment: u64,
    pub timestamp: i64,
}

#[event]
pub struct AccessApproved {
    pub synid: Pubkey,
    pub requester: Pubkey,
    pub payment: u64,
    pub timestamp: i64,
}

#[event]
pub struct AccessDenied {
    pub synid: Pubkey,
    pub requester: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AccessRevoked {
    pub synid: Pubkey,
    pub requester: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct IdentityVerified {
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ReputationUpdated {
    pub owner: Pubkey,
    pub new_score: u16,
    pub timestamp: i64,
}

#[event]
pub struct SynidBurned {
    pub owner: Pubkey,
    pub token_id: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum SynidError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Already minted")]
    AlreadyMinted,
    #[msg("Invalid CID")]
    InvalidCid,
    #[msg("Name too long")]
    NameTooLong,
    #[msg("URI too long")]
    UriTooLong,
    #[msg("CID too long")]
    CidTooLong,
    #[msg("Too many fields")]
    TooManyFields,
    #[msg("Insufficient payment")]
    InsufficientPayment,
    #[msg("Invalid status")]
    InvalidStatus,
    #[msg("Request expired")]
    RequestExpired,
    #[msg("Already revoked")]
    AlreadyRevoked,
    #[msg("Program paused")]
    Paused,
}
