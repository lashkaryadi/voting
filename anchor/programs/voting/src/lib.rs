#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("coUnmi3oBUtwtd9fjeAvSsJssXh5A5xyPbhpewyzRVF");

#[program]
pub mod voting {
    use super::*;

    pub fn initialize_poll(ctx: Context<InitializePoll>, 
                            poll_id: u64,
                            description: String,
                            poll_start: u64,
                            poll_end: u64) -> Result<()> {

        let current_timestamp = Clock::get()?.unix_timestamp as u64;
        
        if poll_end <= current_timestamp {
            return Err(error!(PollError::PollEndInThePast));
        }

        if poll_end == 0 {
            return Err(error!(PollError::InvalidPollEndTimestamp));
        }

        let poll = &mut ctx.accounts.poll;
        poll.poll_id = poll_id;
        poll.description = description;
        poll.poll_start = poll_start;
        poll.poll_end = poll_end;
        poll.candidate_amount = 0;
        poll.total_votes = 0;

        Ok(())
    }

    pub fn initialize_candidate(ctx: Context<InitializeCandidate>, 
                                candidate_name: String, 
                                poll_id: u64) -> Result<()> {

        let candidate = &mut ctx.accounts.candidate;
        candidate.candidate_name = candidate_name;
        candidate.candidate_votes = 0;

        let poll = &mut ctx.accounts.poll;
        poll.candidate_amount += 1;

        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, candidate_name: String, poll_id: u64) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp as u64;
        let poll = &mut ctx.accounts.poll;
        let candidate = &mut ctx.accounts.candidate;
        let voter_key = ctx.accounts.signer.key();

        if poll.voters.contains(&voter_key) {
            return Err(ErrorCode::AlreadyVoted.into());
        }

        if current_timestamp < poll.poll_start || current_timestamp > poll.poll_end {
            return Err(ErrorCode::InvalidVoteTime.into());
        }

        poll.voters.push(voter_key);
        candidate.candidate_votes += 1;
        poll.total_votes += 1;

        msg!("Voted for candidate: {}", candidate.candidate_name);
        msg!("Total votes for {}: {}", candidate.candidate_name, candidate.candidate_votes);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
      init,
      payer = signer,
      space = 8 + Poll::INIT_SPACE,
      seeds = [poll_id.to_le_bytes().as_ref()],
      bump
    )]
    pub poll: Account<'info, Poll>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
      mut,
      seeds = [poll_id.to_le_bytes().as_ref()],
      bump
    )]
    pub poll: Account<'info, Poll>,
    #[account(
        init,
        payer = signer,
        space = 8 + Candidate::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
        bump
    )]
    pub candidate: Account<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, Poll>,
    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
        bump
    )]
    pub candidate: Account<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Poll {
    pub poll_id: u64,
    #[max_len(200)]
    pub description: String,
    pub poll_start: u64,
    pub poll_end: u64,
    pub candidate_amount: u64,
    pub total_votes: u64,
    pub voters: Vec<Pubkey>,
}

#[account]
pub struct Candidate {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_votes: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You have already voted.")]
    AlreadyVoted,
    #[msg("The poll has already ended.")]
    InvalidVoteTime,
}
