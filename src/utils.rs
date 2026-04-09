pub struct Utils;

use ratatui::layout::{Constraint, Rect};

impl Utils {
    pub fn screen_to_board(
        click_x: u16,
        click_y: u16,
        board_width: usize,
        board_height: usize,
        root_area: Rect,
    ) -> Option<(usize, usize)> {
        let pane = Utils::center_rect(root_area, board_width as u16, board_height as u16);

        let min_x = pane.x;
        let min_y = pane.y;
        let max_x = pane.x + pane.width;
        let max_y = pane.y + pane.height;

        if click_x < min_x || click_y < min_y || click_x >= max_x || click_y >= max_y {
            return None;
        }

        Some(((click_x - min_x) as usize, (click_y - min_y) as usize))
    }

    pub fn center_rect(root_area: Rect, width: u16, height: u16) -> Rect {
        root_area.centered(Constraint::Length(width), Constraint::Length(height))
    }
}
