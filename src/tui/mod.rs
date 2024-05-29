use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    terminal::Terminal,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
const TODO_HEADER_BG: Color = tailwind::BLUE.c950;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const COMPLETED_TEXT_COLOR: Color = tailwind::GREEN.c500;

#[derive(Clone)]
pub struct FileViewer {
    state: ListState,
    contents: Vec<String>,
    current_line: usize,
    selected_line: usize,
}

pub struct App {
    current_file: FileViewer,
}

impl App {
    pub fn new() -> Self {
        let current_file: Vec<String> = std::fs::read_to_string("file.cpp")
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect();
        Self {
            current_file: FileViewer {
                contents: current_file,
                current_line: 1,
                selected_line: 1,
                state: ListState::default(),
            },
        }
    }

    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> anyhow::Result<()> {
        loop {
            self.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => self.current_file.next(),
                        KeyCode::Char('k') | KeyCode::Up => self.current_file.previous(),
                        KeyCode::Char('r') => self.current_file.pick_random_line(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> anyhow::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]);
        let [header_area, rest_area, footer_area] = vertical.areas(area);

        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [upper_item_list_area, lower_item_list_area] = vertical.areas(rest_area);

        render_title(header_area, buf);
        self.render_file_viewer(upper_item_list_area, buf);
        self.render_logger(lower_item_list_area, buf);
        render_footer(footer_area, buf);
    }
}

impl App {
    fn render_file_viewer(&mut self, area: Rect, buf: &mut Buffer) {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let outer_block = Block::new()
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title("TODO List")
            .fg(TEXT_COLOR)
            .bg(TODO_HEADER_BG);
        let inner_block = Block::new()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(NORMAL_ROW_COLOR);

        // We get the inner area from outer_block. We'll use this area later to render the table.
        let outer_area = area;
        let inner_area = outer_block.inner(outer_area);

        // We can render the header in outer_area.
        outer_block.render(outer_area, buf);

        let items = self.current_file.clone();
        let items = items.to_item_vec();
        let items = List::new(items)
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(items, inner_area, buf, &mut self.current_file.state);
        //Widget::render(items, inner_area, buf);
    }
    fn render_logger(&mut self, area: Rect, buf: &mut Buffer) {
        // We show the list item's info under the list in this paragraph
        let outer_info_block = Block::new()
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title("Logger")
            .fg(TEXT_COLOR)
            .bg(TODO_HEADER_BG);
        let inner_info_block = Block::new()
            .borders(Borders::NONE)
            .padding(Padding::horizontal(1))
            .bg(NORMAL_ROW_COLOR);

        // This is a similar process to what we did for list. outer_info_area will be used for
        // header inner_info_area will be used for the list info.
        let outer_info_area = area;
        let inner_info_area = outer_info_block.inner(outer_info_area);

        // We can render the header. Inner info will be rendered later
        outer_info_block.render(outer_info_area, buf);

        let info_paragraph = Paragraph::new("Nothing")
            .block(inner_info_block)
            .fg(TEXT_COLOR)
            .wrap(Wrap { trim: false });

        // We can now render the item info
        info_paragraph.render(inner_info_area, buf);
    }
}

fn render_title(area: Rect, buf: &mut Buffer) {
    Paragraph::new("Mdb log converter")
        .bold()
        .centered()
        .render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new("\nUse ↓↑ to move")
        .centered()
        .render(area, buf);
}

impl FileViewer {
    pub fn pick_random_line(&mut self) {
        self.selected_line = self.selected_line + 1;
    }

    pub fn to_item_vec(&self) -> Vec<ListItem> {
        let res = self
            .contents
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let bg_color = if i == self.current_line {
                    ALT_ROW_COLOR
                } else {
                    NORMAL_ROW_COLOR
                };

                let line = if i == self.selected_line {
                    Line::styled(format!(" @ {} {}", i + 1, line), COMPLETED_TEXT_COLOR)
                } else {
                    Line::styled(format!("   {} {}", i + 1, line), TEXT_COLOR)
                };

                ListItem::new(line).bg(bg_color)
            })
            .collect();

        return res;
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.contents.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.current_line,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.contents.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.current_line,
        };
        self.state.select(Some(i));
    }
}
