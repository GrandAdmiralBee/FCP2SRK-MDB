use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::palette::tailwind,
    style::{Color, Modifier, Style, Stylize},
    terminal::Terminal,
    text::Line,
    widgets::{
        Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap,
    },
};
use std::{sync::mpsc, thread};

use log::*;
use tui_logger::*;

pub mod log_list;

use log_list::*;

use crate::mdb_converter::parser::*;

const HEADER_BG: Color = tailwind::BLUE.c950;
const SELECTED_HEADER_BG: Color = tailwind::BLUE.c500;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;
const TEXT_COLOR: Color = tailwind::SLATE.c200;
const COMPLETED_TEXT_COLOR: Color = tailwind::GREEN.c500;

#[derive(Debug)]
pub enum AppEvent {
    Log(String, LogLevel),
    WidgetUp,
    WidgetDown,
    InputFieldChar(char),
    InputFielddBackspace,
    InputFieldComplete,
    Exit,
    StartRender,
    WaitForInput,
    ReplaceFileLine(usize, String),
    InsertFileLine(usize, String),
    Command(String),
    ProceedWithInput,
    JumpLine(usize),
    NewFile(String),
    FileLineDown,
    FileLineUp,
    ReadyToQuit,
}

#[derive(Clone)]
pub struct FileViewer {
    state: ListState,
    contents: Vec<String>,
    current_line: usize,
    selected_line: usize,
}

pub struct InputField {
    current_text: String,
    active: bool,
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
    input_field: InputField,
    ready_to_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            file_viewer: FileViewer {
                contents: Vec::new(),
                current_line: 1,
                selected_line: 1,
                state: ListState::default(),
            },
            input_field: InputField {
                current_text: String::new(),
                active: false,
            },
            current_widget: AppWidget::INPUT_FIELD,
            ready_to_quit: false,
        }
    }

    pub fn run(
        &mut self,
        mut terminal: Terminal<impl Backend>,
        cli: crate::cli::Args,
    ) -> anyhow::Result<()> {
        //let mdb_files = vec!["mdb/fcpasm.mdb".to_string()];
        //let cpp_files = vec!["cpp/FcpAssignmentManager.cpp".to_string()];
        //
        //let cli = crate::cli::Args {
        //    mdb_files,
        //    cpp_files,
        //};

        let (app_sender, rx) = mpsc::channel();
        let (app2parser_sender, app2parser_receiver) = mpsc::channel();
        let tx = app_sender.clone();

        thread::spawn(move || Self::handle_input(tx));
        thread::spawn(move || parser(cli, app_sender.clone(), app2parser_receiver));

        for event in rx {
            match event {
                AppEvent::StartRender => (),
                AppEvent::InputFieldChar(c) => {
                    if self.ready_to_quit {
                        return Ok(());
                    }
                    match self.current_widget {
                        AppWidget::INPUT_FIELD => self.input_field.current_text.push(c),
                        AppWidget::FILE_VIEWER => match c {
                            'k' => self.file_viewer.previous(),
                            'j' => self.file_viewer.next(),
                            _ => (),
                        },
                        _ => (),
                    }
                }
                AppEvent::Exit => break,
                AppEvent::WidgetUp => self.current_widget.up(),
                AppEvent::WidgetDown => self.current_widget.down(),
                AppEvent::InputFielddBackspace => {
                    if self.input_field.current_text.len() > 0 {
                        let _ = self.input_field.current_text.pop();
                    }
                }
                AppEvent::InputFieldComplete => {
                    if self.input_field.current_text.len() > 0 && self.input_field.active {
                        app2parser_sender
                            .send(AppEvent::Command(self.input_field.current_text.clone()))?;
                        self.input_field.current_text.clear();
                        self.input_field.active = false;
                    }
                }
                AppEvent::Log(line, level) => match level {
                    LogLevel::Warn => warn!("{line}"),
                    LogLevel::Trace => trace!("{line}"),
                    LogLevel::Info => info!("{line}"),
                    LogLevel::Error => error!("{line}"),
                },
                AppEvent::WaitForInput => {
                    self.input_field.active = true;
                }
                AppEvent::JumpLine(line) => {
                    self.file_viewer.goto_line(line);
                }
                AppEvent::NewFile(file) => {
                    self.file_viewer.contents = file.lines().map(|x| x.to_string()).collect();
                    self.file_viewer.clear();
                }
                AppEvent::FileLineDown => self.file_viewer.next(),
                AppEvent::FileLineUp => self.file_viewer.previous(),
                AppEvent::ReplaceFileLine(n, line) => self.file_viewer.contents[n - 1] = line,
                AppEvent::InsertFileLine(n, line) => self.file_viewer.contents.insert(n - 1, line),
                AppEvent::ReadyToQuit => {
                    self.ready_to_quit = true;
                    info!("------------ Finished successfully (press any key to quit) ------");
                }
                _ => (),
            }
            self.draw(&mut terminal)?;
        }

        Ok(())
    }

    fn handle_input(tx_event: mpsc::Sender<AppEvent>) -> anyhow::Result<()> {
        tx_event.send(AppEvent::StartRender)?;
        while let Ok(event) = event::read() {
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                            tx_event.send(AppEvent::Exit)?
                        }
                        (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                            tx_event.send(AppEvent::WidgetUp)?
                        }
                        (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                            tx_event.send(AppEvent::WidgetDown)?
                        }
                        (KeyCode::Up, m) => {
                            if m != KeyModifiers::CONTROL {
                                tx_event.send(AppEvent::FileLineUp).unwrap();
                            }
                        }
                        (KeyCode::Down, m) => {
                            if m != KeyModifiers::CONTROL {
                                tx_event.send(AppEvent::FileLineDown).unwrap();
                            }
                        }
                        (KeyCode::Char(c), m) => {
                            if m != KeyModifiers::CONTROL {
                                tx_event.send(AppEvent::InputFieldChar(c))?
                            }
                        }
                        (KeyCode::Enter, _) => tx_event
                            .send(AppEvent::InputFieldComplete)
                            .expect("Crashed here"),
                        (KeyCode::Backspace, _) => tx_event.send(AppEvent::InputFielddBackspace)?,
                        (KeyCode::Esc, _) => tx_event.send(AppEvent::Exit)?,
                        _ => (),
                    }
                }
            }
        }
        Ok(())
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
        self.render_file_viewer(upper_item_list_area, buf);
        self.render_logger(lower_item_list_area, buf);
        self.render_input_field(input_area, buf);
        render_footer(footer_area, buf);
    }
}

impl App {
    fn render_file_viewer(&mut self, area: Rect, buf: &mut Buffer) {
        let bg = match self.current_widget {
            AppWidget::FILE_VIEWER => SELECTED_HEADER_BG,
            _ => HEADER_BG,
        };
        let items = self.file_viewer.clone();
        let items = items.to_item_vec();
        let items = List::new(items)
            .block(Block::bordered().title("Logger"))
            .bg(bg)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(items, area, buf, &mut self.file_viewer.state);
        //Widget::render(items, inner_area, buf);
    }

    fn render_logger(&mut self, area: Rect, buf: &mut Buffer) {
        let bg = match self.current_widget {
            AppWidget::LOGGER => SELECTED_HEADER_BG,
            _ => HEADER_BG,
        };
        // We show the list item's info under the list in this paragraph
        TuiLoggerWidget::default()
            .style_error(Style::default().fg(Color::Red))
            .style_debug(Style::default().fg(Color::Green))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_trace(Style::default().fg(Color::White))
            .block(Block::bordered().title("Logger"))
            .style_info(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(bg))
            .output_timestamp(None)
            .output_separator(':')
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .render(area, buf);
    }

    fn render_input_field(&mut self, area: Rect, buf: &mut Buffer) {
        // We show the list item's info under the list in this paragraph
        let bg = match self.current_widget {
            AppWidget::INPUT_FIELD => SELECTED_HEADER_BG,
            _ => HEADER_BG,
        };
        let input = match self.input_field.active {
            false => self.input_field.current_text.clone(),
            true => format!("> {}", &self.input_field.current_text),
        };
        let info_paragraph = Paragraph::new(input)
            .fg(TEXT_COLOR)
            .bg(bg)
            .block(Block::bordered().title("Command input field"))
            .wrap(Wrap { trim: false });

        // We can now render the item info
        info_paragraph.render(area, buf);
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
    pub fn clear(&mut self) {
        self.current_line = 1;
        self.selected_line = 1;
        self.state.select(None);
    }
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
                    HEADER_BG
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
