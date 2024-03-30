use std::{
    fs::create_dir_all,
    io::{self, Stdout},
    path::Path,
};

use directories::ProjectDirs;

use _2048_rs::{load, print_board, save, Arrow, Board};
use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use rand::rngs::ThreadRng;
use ratatui::prelude::*;

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;

    // if project specified directory is not avaliable,
    // just place saves at ./saves/saveX
    let project_dir = ProjectDirs::from("moe", "Meowkatee", "2048-rs");
    let data_dir = project_dir
        .as_ref()
        .map(|proj| proj.data_dir())
        .unwrap_or(&Path::new("saves"));

    let mut rng = rand::thread_rng();

    let mut current_best = std::fs::read(data_dir.join("best"))
        .map(|b| bitcode::decode(&b).expect("invalid best score file"))
        .unwrap_or(0);

    let score = loop {
        let (score, cont) = run(&mut terminal, data_dir, &mut rng, current_best)?;
        current_best = current_best.max(score);
        if !cont {
            break score;
        }
    };
    create_dir_all(data_dir)?;
    std::fs::write(data_dir.join("best"), bitcode::encode(&current_best))?;
    restore_terminal(&mut terminal)?;
    println!("score: {score}");
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    data_dir: &Path,
    rng: &mut ThreadRng,
    prev_best: u64,
) -> Result<(u64, bool)> {
    let mut board = Board::new(rng);
    let mut new_best = prev_best;

    loop {
        new_best = new_best.max(board.score());
        print_board(board, terminal, board.is_lost(rng), prev_best, new_best)?;
        let input = crossterm::event::read()?;
        match input {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q' | 'Q'),
                kind: KeyEventKind::Press,
                ..
            }) => break Ok((board.score(), false)),
            Event::Key(KeyEvent {
                code: KeyCode::Char('s' | 'S'),
                kind: KeyEventKind::Release,
                ..
            }) => {
                terminal.clear()?;
                save(board, data_dir)?;
                terminal.clear()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l' | 'L'),
                kind: KeyEventKind::Release,
                ..
            }) => {
                terminal.clear()?;
                load(&mut board, data_dir)?;
                terminal.clear()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('r' | 'R'),
                kind: KeyEventKind::Release,
                ..
            }) => break Ok((board.score(), true)),
            Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let Ok(direction) = Arrow::try_from(code) else {
                    continue;
                };
                board.play_changed(direction, rng);
            }
            _ => continue,
        }
    }
}
