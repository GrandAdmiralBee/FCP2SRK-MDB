use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::Backend,
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

const HEADER_BG: Color = tailwind::BLUE.c950;
const SELECTED_HEADER_BG: Color = tailwind::BLUE.c500;
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

pub struct Logger {
    contents: Vec<String>,
}

pub struct InputField {
    current_text: String,
}

#[derive(Clone, Copy)]
pub enum AppWidget {
    FILE_VIEWER,
    LOGGER,
    INPUT_FIELD,
}

impl AppWidget {
    pub fn down(&mut self) {
        use AppWidget::*;
        *self = match self {
            FILE_VIEWER => LOGGER,
            LOGGER => INPUT_FIELD,
            INPUT_FIELD => INPUT_FIELD,
        };
    }
    pub fn up(&mut self) {
        use AppWidget::*;
        *self = match self {
            FILE_VIEWER => FILE_VIEWER,
            LOGGER => FILE_VIEWER,
            INPUT_FIELD => LOGGER,
        };
    }
}

pub struct App {
    current_widget: AppWidget,
    file_viewer: FileViewer,
    logger: Logger,
    input_field: InputField,
    command: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let current_file: Vec<String> = std::fs::read_to_string("file.cpp")
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect();
        Self {
            file_viewer: FileViewer {
                contents: current_file,
                current_line: 1,
                selected_line: 1,
                state: ListState::default(),
            },
            logger: Logger {
                contents: Vec::new(),
            },
            input_field: InputField {
                current_text: String::new(),
            },
            current_widget: AppWidget::INPUT_FIELD,
            command: None,
        }
    }

    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> anyhow::Result<()> {
        loop {
            self.command = None;
            self.draw(&mut terminal)?;
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Up => self.current_widget.up(),
                        KeyCode::Down => self.current_widget.down(),
                        KeyCode::Char(c) => match self.current_widget {
                            AppWidget::INPUT_FIELD => self.input_field.current_text.push(c),
                            AppWidget::FILE_VIEWER => match c {
                                'k' => self.file_viewer.previous(),
                                'j' => self.file_viewer.next(),
                                _ => (),
                            },
                            AppWidget::LOGGER => {}
                        },
                        KeyCode::Enter => match self.current_widget {
                            AppWidget::INPUT_FIELD => {
                                self.command = Some(self.input_field.current_text.clone());
                                self.logger
                                    .contents
                                    .push(self.input_field.current_text.clone());
                                self.input_field.current_text.clear()
                            }
                            _ => (),
                        },
                        KeyCode::Backspace => match self.current_widget {
                            AppWidget::INPUT_FIELD => {
                                self.input_field.current_text.pop();
                            }
                            _ => (),
                        },
                        KeyCode::Tab => match self.current_widget {
                            AppWidget::INPUT_FIELD => self.input_field.current_text.push('\t'),
                            _ => (),
                        },
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

        let vertical = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ]);
        let [upper_item_list_area, lower_item_list_area, input_area] = vertical.areas(rest_area);

        render_title(header_area, buf);
        self.render_file_viewer(upper_item_list_area, buf, self.current_widget);
        self.render_logger(lower_item_list_area, buf, self.current_widget);
        self.render_input_field(input_area, buf, self.current_widget);
        render_footer(footer_area, buf);
    }
}

impl App {
    fn render_file_viewer(&mut self, area: Rect, buf: &mut Buffer, wid: AppWidget) {
        // We create two blocks, one is for the header (outer) and the other is for list (inner).
        let outer_block = Block::new()
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title("File Viewer")
            .fg(TEXT_COLOR)
            .bg(match wid {
                AppWidget::FILE_VIEWER => SELECTED_HEADER_BG,
                _ => HEADER_BG,
            });
        let inner_block = Block::new()
            .borders(Borders::NONE)
            .fg(TEXT_COLOR)
            .bg(NORMAL_ROW_COLOR);

        // We get the inner area from outer_block. We'll use this area later to render the table.
        let outer_area = area;
        let inner_area = outer_block.inner(outer_area);

        // We can render the header in outer_area.
        outer_block.render(outer_area, buf);

        let items = self.file_viewer.clone();
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
        StatefulWidget::render(items, inner_area, buf, &mut self.file_viewer.state);
        //Widget::render(items, inner_area, buf);
    }

    fn render_logger(&mut self, area: Rect, buf: &mut Buffer, wid: AppWidget) {
        // We show the list item's info under the list in this paragraph
        let outer_info_block = Block::new()
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title("Logger")
            .fg(TEXT_COLOR)
            .bg(match wid {
                AppWidget::LOGGER => SELECTED_HEADER_BG,
                _ => HEADER_BG,
            });
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

        let log_list: Vec<ListItem> = self
            .logger
            .contents
            .iter()
            .enumerate()
            .map(|(x, l)| {
                ListItem::new(Line::styled(l, TEXT_COLOR)).bg(
                    if x == self.logger.contents.len() - 1 {
                        ALT_ROW_COLOR
                    } else {
                        NORMAL_ROW_COLOR
                    },
                )
            })
            .collect();

        let items = List::new(log_list)
            .block(inner_info_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        Widget::render(items, inner_info_area, buf);
    }

    fn render_input_field(&mut self, area: Rect, buf: &mut Buffer, wid: AppWidget) {
        // We show the list item's info under the list in this paragraph
        let outer_info_block = Block::new()
            .borders(Borders::NONE)
            .title_alignment(Alignment::Center)
            .title("Input field")
            .fg(TEXT_COLOR)
            .bg(match wid {
                AppWidget::INPUT_FIELD => SELECTED_HEADER_BG,
                _ => HEADER_BG,
            });
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

        let input = self.input_field.current_text.as_str();
        let info_paragraph = Paragraph::new(input)
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
    pub fn goto_line(&mut self, line: usize) {
        if line > self.contents.len() {
            self.selected_line = self.contents.len();
            self.current_line = self.contents.len();
        } else {
            self.selected_line = line;
            self.current_line = line;
        }
        self.select_line();
    }

    pub fn to_item_vec(&self) -> Vec<ListItem> {
        let res = self
            .contents
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let bg_color = if i + 1 == self.current_line {
                    ALT_ROW_COLOR
                } else {
                    NORMAL_ROW_COLOR
                };

                let line = if i + 1 == self.selected_line {
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

    fn select_line(&mut self) {
        self.state.select(Some(self.selected_line - 1));
    }
}
