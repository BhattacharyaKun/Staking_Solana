use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;
use anchor_spl::token;

declare_id!("BRkou4CPQ7mm3ebS9RsEk1A6kKgFdbrrTASzfUqe5RNd");

#[program]
pub mod backend_staking_anchor 
{
    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> 
    {
        let user_info = &mut ctx.accounts.user_info;
        let staking_info = &mut ctx.accounts.staking_info;

        if !user_info.is_initialized
        {
            user_info.is_initialized = true;
            user_info.point_balance = 0;
            user_info.active_stake = 0;
        }

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_account = Transfer 
        { 
            from: ctx.accounts.user_nft_account.to_account_info(), 
            to: ctx.accounts.pda_nft_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info()
        };

        let token_transfer_context = CpiContext::new(cpi_program, cpi_account);
        token::transfer(token_transfer_context, 1)?;

        staking_info.mint = ctx.accounts.mint.key();
        staking_info.staker = ctx.accounts.user.key();
        staking_info.stake_start_time = Clock::get().unwrap().unix_timestamp as u64;
        staking_info.last_stake_redeem = Clock::get().unwrap().unix_timestamp as u64;
        staking_info.staked = true;

        user_info.active_stake = user_info.active_stake.checked_add(1).unwrap();

        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> 
    {        
        let user_info = &mut ctx.accounts.user_info;
        let staking_info = &mut ctx.accounts.staking_info;

        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        let amount = current_time - staking_info.last_stake_redeem;
        staking_info.last_stake_redeem = current_time;

        user_info.point_balance = user_info.point_balance.checked_add(amount).unwrap();

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> 
    {
        let user_info = &mut ctx.accounts.user_info;
        let staking_info = &mut ctx.accounts.staking_info;

        let auth_bump = *ctx.bumps.get("staking_info").unwrap();
        let seeds = &[b"stake_info".as_ref(), &ctx.accounts.user.key().to_bytes(), &ctx.accounts.mint.key().to_bytes(), &[auth_bump]];
        
        let signer = &[&seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.pda_nft_account.to_account_info(),
            to: ctx.accounts.user_nft_account.to_account_info(),
            authority: staking_info.to_account_info(),
        };

        let token_transfer_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(token_transfer_context, 1)?;

        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        let amount = current_time - staking_info.last_stake_redeem;
        staking_info.last_stake_redeem = current_time;

        user_info.point_balance = user_info.point_balance.checked_add(amount).unwrap();

        ctx.accounts.staking_info.staked = false;

        user_info.active_stake = user_info.active_stake.checked_sub(1).unwrap();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> 
{
    #[account(init_if_needed, payer = user, space = std::mem::size_of::<UserInfo>() + 8, seeds=[b"user", user.key().as_ref(), mint.key().as_ref()], bump)]
    pub user_info: Account<'info, UserInfo>,
    
    #[account(init_if_needed, payer = user, space = std::mem::size_of::<UserStakeInfo>() + 8, seeds=[b"stake_info", user.key().as_ref(), mint.key().as_ref()], bump)]
    pub staking_info: Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, constraint = user_nft_account.owner.key() == user.key(), constraint = user_nft_account.mint == mint.key(), constraint = user_nft_account.amount == 1)]
    pub user_nft_account: Account<'info, TokenAccount>,

    #[account(init_if_needed, payer = user, associated_token::mint = mint, associated_token::authority = staking_info)]
    pub pda_nft_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct Redeem<'info> 
{
    #[account(mut, seeds=[b"user", user.key().as_ref(), mint.key().as_ref()], bump)]
    pub user_info: Account<'info, UserInfo>,
    
    #[account(mut, seeds=[b"stake_info", user.key().as_ref(), mint.key().as_ref()], bump)]
    pub staking_info: Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Unstake<'info> 
{
    #[account(mut, seeds=[b"user", user.key().as_ref()], bump)]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut, seeds=[b"stake_info", user.key().as_ref(), mint.key().as_ref()], bump, constraint = user.key() == staking_info.staker, close = user)]
    pub staking_info: Account<'info, UserStakeInfo>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, constraint = user_nft_account.owner.key() == user.key(), constraint = user_nft_account.mint == mint.key(), constraint = user_nft_account.amount == 0)]
    pub user_nft_account: Account<'info, TokenAccount>,

    #[account(mut, constraint = pda_nft_account.owner == staking_info.key(), constraint = pda_nft_account.mint == mint.key(), constraint = pda_nft_account.amount == 1)]
    pub pda_nft_account: Account<'info, TokenAccount>,

    #[account(constraint = staking_info.mint == mint.key())]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserInfo
{
    is_initialized: bool,
    point_balance: u64,
    active_stake: u16
}

#[account]
pub struct UserStakeInfo
{
    staker: Pubkey,
    mint: Pubkey,
    stake_start_time: u64,
    last_stake_redeem: u64,
    staked: bool
}