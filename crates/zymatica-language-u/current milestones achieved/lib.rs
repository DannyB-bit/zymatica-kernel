use anchor_lang::prelude::*;

declare_id!("2is5Q4rPBpZa2RUCXP7FFdHJUYSVNcW5iTxNuf5mSccy");

#[program]
pub mod solana_cuneiform_anchor {
    use super::*;

    /// Initializes the global program state containing the admin, treasury, and protocol fee.
    pub fn initialize_program(
        ctx: Context<InitializeProgram>,
        treasury: Pubkey,
        fee_lamports: u64,
    ) -> Result<()> {
        let state = &mut ctx.accounts.program_state;
        state.admin = ctx.accounts.admin.key();
        state.treasury = treasury;
        state.fee_lamports = fee_lamports;

        msg!("Program state initialized successfully.");
        msg!("Admin: {}", state.admin);
        msg!("Treasury: {}", state.treasury);
        msg!("Protocol Fee (lamports): {}", state.fee_lamports);
        Ok(())
    }

    /// Allows the admin to update treasury address and protocol fees.
    pub fn update_program_state(
        ctx: Context<UpdateProgramState>,
        new_treasury: Option<Pubkey>,
        new_fee_lamports: Option<u64>,
    ) -> Result<()> {
        let state = &mut ctx.accounts.program_state;
        
        if let Some(t) = new_treasury {
            state.treasury = t;
            msg!("Treasury updated to: {}", t);
        }
        if let Some(f) = new_fee_lamports {
            state.fee_lamports = f;
            msg!("Protocol fee updated to: {} lamports", f);
        }

        Ok(())
    }

    /// Registers a new Cuneiform-U semantic coordinate state and collects the protocol fee.
    pub fn register_coordinates(
        ctx: Context<RegisterCoordinates>,
        session_id: [u8; 16],
        coords: [u8; 6],
        merkle_root: [u8; 32],
    ) -> Result<()> {
        let state = &ctx.accounts.program_state;
        
        // 1. Validate that the correct treasury account is passed in keys
        require_keys_eq!(
            ctx.accounts.treasury.key(),
            state.treasury,
            ErrorCode::InvalidTreasury
        );

        // 2. Perform CPI transfer of the protocol fee to the treasury wallet
        if state.fee_lamports > 0 {
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
            );
            anchor_lang::system_program::transfer(cpi_context, state.fee_lamports)?;
            msg!("Collected protocol fee: {} lamports", state.fee_lamports);
        }

        // 3. Register the Cuneiform coordinates state
        let record = &mut ctx.accounts.coordinate_record;
        record.authority = ctx.accounts.authority.key();
        record.session_id = session_id;
        record.coords = coords;
        record.merkle_root = merkle_root;
        record.timestamp = Clock::get()?.unix_timestamp;
        record.bump = ctx.bumps.coordinate_record;

        msg!("Language-U coordinates registered successfully!");
        msg!("Coordinates: Domain={}, Subdomain={}, Modality={}, Polarity={}, Strength={}, Depth={}", 
            coords[0], coords[1], coords[2], coords[3], coords[4], coords[5]);

        Ok(())
    }

    /// Updates the coordinates for an existing session record (free of protocol fee).
    pub fn update_coordinates(
        ctx: Context<UpdateCoordinates>,
        coords: [u8; 6],
        merkle_root: [u8; 32],
    ) -> Result<()> {
        let record = &mut ctx.accounts.coordinate_record;
        
        record.coords = coords;
        record.merkle_root = merkle_root;
        record.timestamp = Clock::get()?.unix_timestamp;

        msg!("Language-U coordinates updated successfully.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + ProgramState::INIT_SPACE,
        seeds = [b"state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateProgramState<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump,
        has_one = admin
    )]
    pub program_state: Account<'info, ProgramState>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(session_id: [u8; 16])]
pub struct RegisterCoordinates<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + CoordinateRecord::INIT_SPACE,
        seeds = [b"cuneiform", authority.key().as_ref(), &session_id],
        bump
    )]
    pub coordinate_record: Account<'info, CoordinateRecord>,

    /// The global program state config to fetch fee and treasury details
    #[account(
        seeds = [b"state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,

    /// The treasury account that receives the protocol fee
    #[account(mut)]
    /// CHECK: Validated against state.treasury in program logic
    pub treasury: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateCoordinates<'info> {
    #[account(
        mut,
        seeds = [b"cuneiform", authority.key().as_ref(), &coordinate_record.session_id],
        bump = coordinate_record.bump,
        has_one = authority
    )]
    pub coordinate_record: Account<'info, CoordinateRecord>,

    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct ProgramState {
    pub admin: Pubkey,         // 32 bytes
    pub treasury: Pubkey,      // 32 bytes
    pub fee_lamports: u64,     // 8 bytes
}

#[account]
#[derive(InitSpace)]
pub struct CoordinateRecord {
    pub authority: Pubkey,       // 32 bytes
    pub session_id: [u8; 16],    // 16 bytes
    pub coords: [u8; 6],         // 6 bytes (DOMAIN, SUBDOMAIN, MODALITY, POLARITY, STRENGTH, DEPTH)
    pub merkle_root: [u8; 32],   // 32 bytes (attestation seal)
    pub timestamp: i64,          // 8 bytes
    pub bump: u8,                // 1 byte
}

#[error_code]
pub enum ErrorCode {
    #[msg("The provided treasury account does not match the configured program state treasury.")]
    InvalidTreasury,
}
