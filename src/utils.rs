pub struct Utils;

use ratatui::layout::{Constraint, Flex, Layout, Rect};

impl Utils {
    pub fn screen_to_board(
        click_x: u16,
        click_y: u16,
        board_width: usize,
        board_height: usize,
        root_area: Rect,
    ) -> Option<(usize, usize)> {
        let pane = Utils::center(root_area, board_width as u16, board_height as u16);

        let min_x = pane.x;
        let min_y = pane.y;
        let max_x = pane.x + pane.width;
        let max_y = pane.y + pane.height;

        if click_x < min_x || click_y < min_y || click_x >= max_x || click_y >= max_y {
            return None;
        }

        Some(((click_x - min_x) as usize, (click_y - min_y) as usize))
    }

    pub fn center(root_area: Rect, width: u16, height: u16) -> Rect {
        root_area.centered(Constraint::Length(width), Constraint::Length(height))
    }

    pub fn center_right(root: Rect, width: u16, height: u16) -> Rect {
        let [h] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::End)
            .areas(root);

        let [v] = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Center)
            .areas(h);

        v
    }
}
