use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::symbols::line;
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};

pub struct ToggleButtonState<'a> {
    pub text: &'a str,
    pub selected: bool,
}

pub struct ToggleButtons<'a> {
    block: Option<Block<'a>>,
    /// Text style
    style: Style,
    selected_style: Style,
    /// ToggleButtons State
    state: Vec<ToggleButtonState<'a>>,
    /// Visible borders
    borders: Borders,
    /// Border style
    border_style: Style,
}

impl<'a> ToggleButtons<'a> {
    pub fn new(state: Vec<ToggleButtonState<'a>>) -> ToggleButtons<'a> {
        ToggleButtons {
            block: None,
            style: Default::default(),
            selected_style: Default::default(),
            state,
            borders: Borders::NONE,
            border_style: Default::default(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> ToggleButtons<'a> {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for ToggleButtons<'a> {
    fn draw(&mut self, textbox_area: Rect, buf: &mut Buffer) {
        let textbox_area = match self.block {
            Some(ref mut b) => {
                b.draw(textbox_area, buf);
                b.inner(textbox_area)
            }
            None => textbox_area,
        };
        if textbox_area.width < 2 || textbox_area.height < 2 {
            return;
        }

        self.background(textbox_area, buf, self.style.bg);

        // Sides
        if self.borders.intersects(Borders::LEFT) {
            for y in textbox_area.top()..textbox_area.bottom() {
                buf.get_mut(textbox_area.left(), y)
                    .set_symbol(line::VERTICAL)
                    .set_style(self.border_style);
            }
        }
        if self.borders.intersects(Borders::TOP) {
            for x in textbox_area.left()..textbox_area.right() {
                buf.get_mut(x, textbox_area.top())
                    .set_symbol(line::HORIZONTAL)
                    .set_style(self.border_style);
            }
        }
        if self.borders.intersects(Borders::RIGHT) {
            let x = textbox_area.right() - 1;
            for y in textbox_area.top()..textbox_area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(line::VERTICAL)
                    .set_style(self.border_style);
            }
        }
        if self.borders.intersects(Borders::BOTTOM) {
            let y = textbox_area.bottom() - 1;
            for x in textbox_area.left()..textbox_area.right() {
                buf.get_mut(x, y)
                    .set_symbol(line::HORIZONTAL)
                    .set_style(self.border_style);
            }
        }

        // Corners
        if self.borders.contains(Borders::LEFT | Borders::TOP) {
            buf.get_mut(textbox_area.left(), textbox_area.top())
                .set_symbol(line::TOP_LEFT)
                .set_style(self.border_style);
        }
        if self.borders.contains(Borders::RIGHT | Borders::TOP) {
            buf.get_mut(textbox_area.right() - 1, textbox_area.top())
                .set_symbol(line::TOP_RIGHT)
                .set_style(self.border_style);
        }
        if self.borders.contains(Borders::LEFT | Borders::BOTTOM) {
            buf.get_mut(textbox_area.left(), textbox_area.bottom() - 1)
                .set_symbol(line::BOTTOM_LEFT)
                .set_style(self.border_style);
        }
        if self.borders.contains(Borders::RIGHT | Borders::BOTTOM) {
            buf.get_mut(textbox_area.right() - 1, textbox_area.bottom() - 1)
                .set_symbol(line::BOTTOM_RIGHT)
                .set_style(self.border_style);
        }

        let text_position_x = if self.borders.contains(Borders::LEFT) {
            textbox_area.left() + 1
        } else {
            textbox_area.left()
        };
        let text_position_y = if self.borders.contains(Borders::TOP) {
            textbox_area.top() + 1
        } else {
            textbox_area.top()
        };

        let textbox_width = textbox_area.width
            - u16::from(self.borders.contains(Borders::LEFT))
            - u16::from(self.borders.contains(Borders::RIGHT));

        let textbox_height = textbox_area.height
            - u16::from(self.borders.contains(Borders::TOP))
            - u16::from(self.borders.contains(Borders::BOTTOM));

        let texts: Vec<Text> = self
            .state
            .iter()
            .enumerate()
            .map(|(index, button_state)| {
                let ToggleButtonState { text, selected } = button_state;
                let selection_alphabetic = ('A' as u8 + index as u8) as char;
                if *selected {
                    Text::styled(
                        format!("[x]{}. {}\n", selection_alphabetic, text),
                        self.selected_style,
                    )
                } else {
                    Text::styled(
                        format!("[ ]{}. {}\n", selection_alphabetic, text),
                        self.style,
                    )
                }
            })
            .collect();

        let para_rect = Rect::new(
            text_position_x,
            text_position_y,
            textbox_width,
            textbox_height,
        );
        Paragraph::new(texts.iter()).wrap(true).draw(para_rect, buf);
    }
}
