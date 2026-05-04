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

    pub fn top_right(root: Rect, width: u16, height: u16) -> Rect {
        let [h] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::End)
            .areas(root);

        let [v] = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Start)
            .areas(h);

        v
    }

    pub fn top_left(root: Rect, width: u16, height: u16) -> Rect {
        let [h] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Start)
            .areas(root);

        let [v] = Layout::vertical([Constraint::Length(height)])
            .flex(Flex::Start)
            .areas(h);

        v
    }

    pub fn num_to_big_text(number: isize) -> String {
        let mut num = number;
        let mut big_text: [[char; 9]; 3] = [[' '; 9]; 3];
        /* ═╠║╣╔╗╚╝╬╦╩
        ╔═╗
        ╠═╣
        ╚═╝
        */
        let minus = "   ═══   ";
        let map: [&str; 10] = [
            "╔═╗║ ║╚═╝",
            "╔╗  ║ ═╩═",
            "╔═╗╔═╝╚═╝",
            "╔═╗ ╠╣╚═╝",
            "╦ ╦╚═╣  ╩",
            "╔═╗╚═╗╚═╝",
            "╔═╗╠═╗╚═╝",
            "╔═╗  ║  ╩",
            "╔═╗╠═╣╚═╝",
            "╔═╗╚═╣╚═╝",
        ];

        for idx in (0..3).rev() {
            let units_place: usize = (num % 10).abs().try_into().unwrap();
            num = num / 10;
            let offset = idx * 3;
            for row in 0..3 {
                for col in 0..3 {
                    big_text[row][offset + col] =
                        map[units_place].chars().nth(row * 3 + col).unwrap()
                }
            }
        }

        let mut final_text: String = String::from("");

        if number < 0 {
            let offset = if number.abs() < 10 { 3 } else { 0 };
            for row in 0..3 {
                for col in 0..3 {
                    big_text[row][offset + col] = minus.chars().nth(row * 3 + col).unwrap()
                }
            }
        }

        for row in 0..3 {
            for col in 0..9 {
                final_text.push(big_text[row][col]);
            }
            final_text.push('\n');
        }

        final_text
    }
}
