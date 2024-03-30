use std::io::Stdout;
use std::num::NonZeroU8;

use ratatui::prelude::*;
use ratatui::widgets::block::Title;
use ratatui::widgets::*;

use anyhow::Result;

use crate::Board;

fn color_of(state: NonZeroU8) -> Color {
    match state.get() % 8 {
        0 => Color::Rgb(255, 0, 77),
        1 => Color::Rgb(255, 163, 0),
        2 => Color::Rgb(255, 240, 36),
        3 => Color::Rgb(0, 231, 86),
        4 => Color::Rgb(41, 173, 255),
        5 => Color::Rgb(131, 118, 156),
        6 => Color::Rgb(255, 119, 168),
        7 => Color::Rgb(255, 119, 168),
        _ => unreachable!(),
    }
}

fn board_to_table(board: Board, prev_best: u64, new_best: u64) -> Table<'static> {
    let title = Title::from(format!(
        "{}Score: {}/{new_best}",
        if board.score() > prev_best { "*" } else { "" },
        board.score
    ))
    .alignment(Alignment::Right);
    Table::new(
        board
            .board
            .map(|row| Row::new(row.map(cell_to_widget)).height(CELL_HEIGHT)),
            &[Constraint::Percentage(25); 4]
    )
    .column_spacing(0)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::new().bg(Color::DarkGray))
            .padding(Padding::new(BOARD_HORIZON_PAD + 1, BOARD_HORIZON_PAD, 1, 1)),
    )
}

fn cell_to_widget(cell: Option<NonZeroU8>) -> Cell<'static> {
    let text = Text::raw(format!(
        "\n {}",
        cell.map(|i| 2_u64.saturating_pow(i.get() as _).to_string())
            .unwrap_or_default(),
    ));
    Cell::from(text).style(
        Style::new()
            .bg(Color::Black)
            .fg(cell.map(color_of).unwrap_or(Color::Black)),
    )
}

pub fn print_board(
    board: Board,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    lost: bool,
    prev_best: u64,
    new_best: u64,
) -> Result<()> {
    terminal.draw(|frame| {
        let table = board_to_table(board, prev_best, new_best);

        if lost {
            let graph = Paragraph::new("You lost!").fg(Color::Red);
            frame.render_widget(
                graph,
                Rect {
                    x: 0,
                    y: BOARD_HEIGHT + 1,
                    width: BOARD_WIDTH,
                    height: 1,
                },
            );
        }

        frame.render_widget(
            table,
            Rect {
                x: 0,
                y: 0,
                width: BOARD_WIDTH,
                height: BOARD_HEIGHT,
            },
        );
    })?;
    Ok(())
}

const BOARD_HORIZON_PAD: u16 = 2;
const BOARD_WIDTH: u16 = BOARD_HORIZON_PAD * 2 + 4 * 7;
const CELL_HEIGHT: u16 = 3;
const BOARD_HEIGHT: u16 = 4 + CELL_HEIGHT * 4;
