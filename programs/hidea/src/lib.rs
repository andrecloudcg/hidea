use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::clock::Clock;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod hidea {
    use super::*;

    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        mode: u8,         // 0 = PvP, 1 = PvE
        bet_amount: u64,
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;

        // Transfert des tokens (bet) de player1 vers le vault
        if bet_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.player1_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.player1.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            token::transfer(
                CpiContext::new(cpi_program, cpi_accounts),
                bet_amount
            )?;
        }

        // Initialisation des joueurs
        game.player1 = ctx.accounts.player1.key();
        game.player2 = if mode == 1 { Pubkey::default() } else { Pubkey::default() };
        game.mode = mode;
        game.bet_amount = bet_amount;

        // Plateau initial
        game.board = default_board();
        game.turn = game.player1;
        game.winner = None;
        game.is_active = true;

        Ok(())
    }

    pub fn play_move(
        ctx: Context<PlayMove>,
        from_x: u8,
        from_y: u8,
        to_x: u8,
        to_y: u8,
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        let player = ctx.accounts.player.key();

        require!(game.is_active, GameError::GameFinished);
        require!(game.turn == player, GameError::NotYourTurn);

        // Déplacement simple
        let piece = game.board[from_y as usize][from_x as usize];
        require!(piece != 0, GameError::InvalidMove);

        game.board[to_y as usize][to_x as usize] = piece;
        game.board[from_y as usize][from_x as usize] = 0;

        // Changement de tour
        if game.mode == 0 {
            game.turn = if game.turn == game.player1 { game.player2 } else { game.player1 };
        } else {
            play_ai_move(game)?;
            game.turn = game.player1;
        }

        // Vérifier fin de partie
        let winner = check_winner(&game.board);
        if winner == 1 {
            game.winner = Some(game.player1);
            game.is_active = false;
        } else if winner == 2 {
            game.winner = Some(game.player2);
            game.is_active = false;
        }

        Ok(())
    }
}

// ------------------------------------------------------------
// Contexts
// ------------------------------------------------------------
#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(init, payer = player1, space = 8 + GameAccount::LEN)]
    pub game: Account<'info, GameAccount>,

    #[account(mut)]
    pub player1: Signer<'info>,

    #[account(mut)]
    pub player1_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlayMove<'info> {
    #[account(mut)]
    pub game: Account<'info, GameAccount>,

    pub player: Signer<'info>,
}

// ------------------------------------------------------------
// GameAccount
// ------------------------------------------------------------
#[account]
pub struct GameAccount {
    pub player1: Pubkey,
    pub player2: Pubkey,
    pub mode: u8,
    pub bet_amount: u64,
    pub board: [[u8;8];8],
    pub turn: Pubkey,
    pub winner: Option<Pubkey>,
    pub is_active: bool,
}

impl GameAccount {
    pub const LEN: usize = 32 + 32 + 1 + 8 + (8*8) + 32 + 32 + 1;
}

// ------------------------------------------------------------
// Fonctions utilitaires
// ------------------------------------------------------------
fn default_board() -> [[u8;8];8] {
    let mut board = [[0u8;8];8];

    for y in 0..3 {
        for x in 0..8 {
            if (x + y) % 2 == 1 { board[y][x] = 1; }
        }
    }

    for y in 5..8 {
        for x in 0..8 {
            if (x + y) % 2 == 1 { board[y][x] = 2; }
        }
    }

    board
}

fn check_winner(board: &[[u8;8];8]) -> u8 {
    let mut p1 = 0;
    let mut p2 = 0;

    for y in 0..8 {
        for x in 0..8 {
            match board[y][x] {
                1 | 3 => p1 += 1,
                2 | 4 => p2 += 1,
                _ => {}
            }
        }
    }

    if p1 == 0 { 2 }
    else if p2 == 0 { 1 }
    else { 0 }
}

fn play_ai_move(game: &mut GameAccount) -> Result<()> {
    for y in 0..8 {
        for x in 0..8 {
            let piece = game.board[y][x];
            if piece == 2 {
                if y > 0 && x > 0 && game.board[y-1][x-1] == 0 {
                    game.board[y-1][x-1] = piece;
                    game.board[y][x] = 0;
                    return Ok(());
                }
                if y > 0 && x < 7 && game.board[y-1][x+1] == 0 {
                    game.board[y-1][x+1] = piece;
                    game.board[y][x] = 0;
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

// ------------------------------------------------------------
// Erreurs
// ------------------------------------------------------------
#[error_code]
pub enum GameError {
    #[msg("Ce n'est pas votre tour.")]
    NotYourTurn,
    #[msg("La partie est terminée.")]
    GameFinished,
    #[msg("Coup invalide.")]
    InvalidMove,
}
