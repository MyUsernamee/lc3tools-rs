use std::ops::{AddAssign, SubAssign};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use keybinds::Keybinds;
use ratatui::prelude::*;
use ratatui::text::ToSpan;
use ratatui::widgets::{Block, BorderType, Borders, Row, Table, TableState};
use ratatui::{DefaultTerminal, Frame, text::Line};
use serde::Deserialize;

use crate::LC3Simulator;

pub mod nvim;

#[derive(PartialEq, Clone, Copy)]
pub enum WindowSelection {
    Memory = 0,
    Output = 1,
    State = 2,
}

impl Into<usize> for WindowSelection {
    fn into(self) -> usize {
        match self {
            WindowSelection::Memory => 0,
            WindowSelection::Output => 1,
            WindowSelection::State => 2,
        } 
    }
}
impl From<usize> for WindowSelection {
    fn from(value: usize) -> Self {
        match value {
            0 => WindowSelection::Memory,
            1 => WindowSelection::Output,
            2 => WindowSelection::State,
            _ => WindowSelection::Memory
        }
    }
}

impl AddAssign<usize> for WindowSelection {
    fn add_assign(&mut self, rhs: usize) {
        let a: usize = *self as usize;
        let a = (a + rhs) % 3;
        *self = a.into();
    }
}
impl SubAssign<usize> for WindowSelection {
    fn sub_assign(&mut self, rhs: usize) {
        let a: usize = *self as usize;
        let a = ((a + 3) - rhs) % 3;
        *self = a.into();
    }
}

#[derive(Deserialize, Clone, Copy)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    CycleWindow,
    ReverseCycleWindow,
    Quit,
}

#[derive(Clone, Copy)]
pub enum Mode {
    Insert,
    Normal,
}

#[derive(Deserialize)]
pub struct Config {
    pub keybinds: Keybinds<Action>,
}

pub struct Debugger {
    sim: LC3Simulator,
    current_window: WindowSelection,
    config: Config,
    mode: Mode,
    memory_table_state: TableState
}

const DEFAULT_KEYBINDS: &str = r#"
[keybinds]
"l" = "Right"
"j" = "Down"
"k" = "Up"
"h" = "Left"

"Tab" = "CycleWindow"
"Shift+Tab" = "ReverseCycleWindow"

"Escape" = "Quit"
"#;

impl Default for Debugger {
    fn default() -> Self {
        let config = toml::from_str(DEFAULT_KEYBINDS).expect("Unable to create default keybinds.");
        Self {
            sim: LC3Simulator::with_os(),
            current_window: WindowSelection::Output,
            config,
            mode: Mode::Normal,
            memory_table_state: TableState::default()
        }
    }
}

impl Debugger {
    pub fn get_sim(&self) -> &LC3Simulator {
        &self.sim
    }

    pub fn get_sim_mut(&mut self) -> &mut LC3Simulator {
        &mut self.sim
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        ratatui::run(|t: &mut DefaultTerminal| {
            loop {
                t.draw(|f: &mut Frame| self.render_frame(f))?;
                let event = crossterm::event::read()?;
                let should_exit = self.handle_event(event);
                if should_exit {
                    break Ok(());
                }
            }
        })
    }

    pub fn handle_event(&mut self, event: crossterm::event::Event) -> bool {
        match event {
            crossterm::event::Event::Key(key_event) => {
                return self.handle_key_event(key_event);
            }
            _ => {}
        }
        false
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if let Some(action) = self.config.keybinds.dispatch(&key_event) && key_event.is_press() {
            match (*action, self.mode) {
                (Action::Quit, Mode::Insert) => {self.mode = Mode::Normal;},
                (Action::CycleWindow, Mode::Normal) => {
                    self.current_window += 1;
                }
                (Action::ReverseCycleWindow, Mode::Normal) => {
                    self.current_window -= 1;
                }
                (action, mode) => {
                    match self.current_window {
                        WindowSelection::Memory => self.handle_memory_input(action, mode),
                        WindowSelection::Output => self.handle_output_input(action, mode),
                        WindowSelection::State => self.handle_state_input(action, mode),
                    } 
                }
                _ => {}
            }
        }

        false
    }

    pub fn render_frame(&mut self, frame: &mut Frame) {
        let title = Line::from("LC3 Debugger");
        frame.render_widget(title, frame.area());

        let [memory_area, right] =
            Layout::horizontal(Constraint::from_percentages([50, 50])).areas(frame.area());

        let [state_area, output_area] =
            Layout::vertical(Constraint::from_percentages([50, 50])).areas(right);

        self.render_memory(frame, memory_area);
        self.render_state(frame, state_area);
        self.render_output(frame, output_area);
    }

    fn draw_subwindow(&'_ self, title: String, selected: bool) -> Block<'_> {
        let mut title = Line::from(title);
        if selected {
            title = title.bold().underlined();
        }
        let block = Block::bordered().title(title);

        block
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow(
            "Output".to_string(),
            self.current_window == WindowSelection::Output,
        );
        frame.render_widget(block, area);
    }

    fn render_memory(&mut self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow(
            "Memory".to_string(),
            self.current_window == WindowSelection::Memory,
        );

        let mem_table = Table::default()
            .header(Row::new(vec!["Address", "Value", "Annotation"]));

        frame.render_widget(&block, area);
        frame.render_stateful_widget(mem_table, block.inner(area), &mut self.memory_table_state);
    }

    fn render_state(&self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow(
            "State".to_string(),
            self.current_window == WindowSelection::State,
        );
        frame.render_widget(block, area);
    }

    fn handle_memory_input(&mut self, action: Action, mode: Mode) {
        todo!()
    }

    fn handle_output_input(&mut self, action: Action, mode: Mode) {
        todo!()
    }

    fn handle_state_input(&mut self, action: Action, mode: Mode) {
        todo!()
    }
}
