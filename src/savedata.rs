use super::Board;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use std::{fs::create_dir_all, path::Path, io::{Write, Read}};

pub fn save(board: Board, data_dir: &Path) -> Result<()> {
    println!("\n\nPlease choose slot (0-9):");
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(slot @ '0'..='9'),
            kind: KeyEventKind::Release,
            ..
        }) = crossterm::event::read()?
        {
            let savefile_path = data_dir.join(format!("save{slot}"));
            if savefile_path.exists() {
                println!("Slot{slot} exists. press it's number to overwrite,");
                println!("or any other to cancel.");
                if let Event::Key(KeyEvent {
                    code: KeyCode::Char(ch),
                    kind: KeyEventKind::Press,
                    ..
                }) = crossterm::event::read()?
                {
                    if ch != slot {
                        return Ok(());
                    }
                }
            }

            create_dir_all(data_dir)?;
            let mut savefile = std::fs::OpenOptions::new()
                .read(false)
                .write(true)
                .append(false)
                .create(true)
                .open(savefile_path)?;

            savefile.write_all(&bitcode::encode(&board))?;
            break Ok(());
        }
    }
}

pub fn load(board: &mut Board, data_dir: &Path) -> Result<()> {
    println!("\n\nPlease choose slot (0-9):");
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(slot @ '0'..='9'),
            kind: KeyEventKind::Press,
            ..
        }) = crossterm::event::read()?
        {
            let savefile_path = data_dir.join(format!("save{slot}"));
            if !savefile_path.exists() {
                return Ok(());
            }

            let mut savefile = std::fs::OpenOptions::new()
                .read(true)
                .write(false)
                .append(false)
                .open(savefile_path)?;

            let mut save = Vec::new();
            savefile.read_to_end(&mut save);
            *board = bitcode::decode(&save)?;
            break Ok(());
        }
    }
}
