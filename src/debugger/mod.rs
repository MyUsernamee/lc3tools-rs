use std::ops::{AddAssign, SubAssign};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{JoinHandle};

use crossterm::event::{KeyCode, KeyEvent};
use keybinds::Keybinds;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Row, Table, TableState};
use ratatui::{DefaultTerminal, Frame, text::Line};
use serde::Deserialize;
use crate::{LC3Simulator};
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
    PageUp,
    PageDown,
    CycleWindow,
    ReverseCycleWindow,
    Quit,
    CommandMode,
    Home,
    Step,
    Run,
    Stop,
    ToggleBreakpoint,
    Top,
    Bottom,
    Enter,
}

pub enum KeyPress {
    Action(Action),
    Code(KeyCode)
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Insert,
    Normal,
    Command
}

#[derive(Deserialize)]
pub struct Config {
    pub keybinds: Keybinds<Action>,
}

pub struct MemTableState {
    table_state: TableState,
    address: u16
}

pub struct Debugger {
    sim: Arc<Mutex<LC3Simulator>>,
    current_window: WindowSelection,
    config: Config,
    mode: Mode,
    memory_table_state: MemTableState,
    output: Arc<Mutex<String>>,
    command_buffer: String,
    display_textline: Option<String>,
    execution_thread: Option<JoinHandle<()>>,
    running: Arc<RwLock<bool>>
}

const DEFAULT_KEYBINDS: &str = r#"
[keybinds]
"l" = "Right"
"j" = "Down"
"k" = "Up"
"h" = "Left"
"'" = "Home"
"g" = "Top"
"G" = "Bottom"

"Tab" = "CycleWindow"
"Shift+Tab" = "ReverseCycleWindow"

"Escape" = "Quit"
"Enter" = "Enter"

":" = "CommandMode"
"s" = "Step"
"r" = "Run"
"R" = "Stop"
"b" = "ToggleBreakpoint"
"#;

impl Default for Debugger {
    fn default() -> Self {
        let config = toml::from_str(DEFAULT_KEYBINDS).expect("Unable to create default keybinds.");
        Self {
            sim: Arc::new(Mutex::new(LC3Simulator::with_os())),
            current_window: WindowSelection::Output,
            config,
            mode: Mode::Normal,
            memory_table_state: MemTableState { table_state: TableState::new(), address: 0 },
            output: Arc::default(),
            command_buffer: String::default(),
            display_textline: None,
            execution_thread: None,
            running: Arc::new(RwLock::new(false)),
        }
    }
}

impl Debugger {
    pub fn with_sim(sim: LC3Simulator) -> Self {
        let new = Self {
            sim: Arc::new(Mutex::new(sim)),
            ..Default::default()
        };
        let output_rc = new.output.clone();
        {
            let mut sim = new.sim.lock().unwrap();

            sim.write(1u16<<15, 0xFE04); // Tell the os we are ready for another char.
            sim.add_write_callback(0xFE06, move |sim, value| {
                *(output_rc.lock().unwrap()) += String::from_utf8(vec![value as u8]).unwrap_or("".to_string()).as_str();
                sim.write(1u16<<15, 0xFE04); // Tell the os we are ready for another char.            
            });
        }

        new
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
        if !key_event.is_press() {return false;}

        let mut press: KeyPress = KeyPress::Code(key_event.code);
        if let Some(action) = self.config.keybinds.dispatch(&key_event) && key_event.is_press() {
            press = KeyPress::Action(*action);
        }

        match (press, self.mode) {
            (press, Mode::Command) => {
                match press
                {
                    KeyPress::Action(Action::Enter) => {
                        let ret = self.handle_command(self.command_buffer.clone());
                        self.command_buffer = "".to_string();
                        self.display_textline = None;
                        self.mode = Mode::Normal;
                        return ret;
                    },
                    KeyPress::Code(code) => {
                        if let Some(c) = code.as_char() {
                            self.command_buffer += c.to_string().as_str();
                        }
                        if code.is_backspace() {
                            self.command_buffer.pop();
                        }
                        self.display_textline = Some(format!(":{}", self.command_buffer));
                    }
                    _ => {}
                }
                
            },
            (KeyPress::Action(action), mode) => {
                match (action, mode) {
                    (Action::Quit, Mode::Insert) => {self.mode = Mode::Normal;},
                    (Action::Quit, Mode::Normal) => {return true;}
                    (Action::Step, Mode::Normal) => { self.sim.lock().unwrap().step(); }
                    (Action::Run, Mode::Normal) => { 
                        let sim = self.sim.clone();
                        let running = self.running.clone();
                        *running.write().unwrap() = true;
                        self.execution_thread = Some(std::thread::spawn(move || {
                            while sim.lock().unwrap().step() && *running.read().unwrap() { }
                            *running.write().unwrap() = false;
                        }));
                    }
                    (Action::Stop, Mode::Normal) => {
                        *self.running.write().unwrap() = false;
                        if let Some(thread) = self.execution_thread.take() {
                            if !thread.is_finished() {
                                thread.join().unwrap();
                            }
                        }
                    }
                    (Action::Home, Mode::Normal) => {
                        self.memory_table_state.address = self.sim.lock().unwrap().get_program_counter();
                    }
                    (Action::CommandMode, Mode::Normal) => {
                        self.mode = Mode::Command;
                        self.display_textline = Some(String::default());
                        self.command_buffer = String::default();
                        self.display_textline = Some(format!(":{}", self.command_buffer));
                    }
                    (Action::CycleWindow, Mode::Normal) => {
                        self.current_window += 1;
                    }
                    (Action::ReverseCycleWindow, Mode::Normal) => {
                        self.current_window -= 1;
                    }
                    (Action::Quit, Mode::Command) => {
                        self.mode = Mode::Normal;
                        self.command_buffer.clear();
                        self.display_textline = None;
                    }

                    (action, mode) => {
                        match self.current_window {
                            WindowSelection::Memory => self.handle_memory_input(KeyPress::Action(action), mode),
                            WindowSelection::Output => self.handle_output_input(KeyPress::Action(action), mode),
                            WindowSelection::State => self.handle_state_input(KeyPress::Action(action), mode),
                        } 
                    }
                    _ => { }
                }
            },
            _ => {}
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

        self.render_display_textline(frame);
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

        let text = Text::from(self.output.lock().unwrap().clone());

        frame.render_widget(&block, area);
        frame.render_widget(text, block.inner(area));
    }

    fn render_memory(&mut self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow(
            "Memory".to_string(),
            self.current_window == WindowSelection::Memory,
        );

        let num_visible_rows = block.inner(area).height;
        let selected_index: Option<usize>;

        let rows = {
            let start_address: u16 = self.memory_table_state.address;
            let addresses = (0..num_visible_rows).map(move |x| {
                ((start_address as i32 - num_visible_rows as i32 / 2) + x as i32).rem_euclid(0x10000)
            });
            selected_index = addresses.clone().position(|x| {
                let sim = self.sim.lock().unwrap();
                x as u16 == sim.get_program_counter()
            } );
            let values = addresses.map(|address| {
                let sim = self.sim.lock().unwrap();
                (address, sim.get_memory()[address as usize], sim.get_annotations()[address as usize].clone())
            });
            values.map(|v| { 
                Row::new(vec![format!("0x{:04X}", v.0), format!("0x{:04X}", v.1), v.2.unwrap_or("".to_string())]) 
            })
        };

        self.memory_table_state.table_state.select(selected_index);

        let mem_table = Table::default()
            .header(Row::new(vec!["Address", "Value", "Annotation"]))
            .highlight_symbol(">>")
            .rows(rows);

        let block = self.draw_subwindow(
            "Memory".to_string(),
            self.current_window == WindowSelection::Memory,
        );
        frame.render_widget(&block, area);
        frame.render_stateful_widget(mem_table, block.inner(area), &mut self.memory_table_state.table_state);
    }

    fn render_state(&self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow(
            "State".to_string(),
            self.current_window == WindowSelection::State,
        );
        frame.render_widget(block, area);
    }

    fn handle_memory_input(&mut self, press: KeyPress, mode: Mode) {
        match (press, mode) {
            (KeyPress::Action(Action::Up), Mode::Normal) => {
                self.memory_table_state.address = self.memory_table_state.address.wrapping_sub(1);
            },
            (KeyPress::Action(Action::Down), Mode::Normal) => {
                self.memory_table_state.address = self.memory_table_state.address.wrapping_add(1);
            },
            (KeyPress::Action(Action::Top), Mode::Normal) => {
                self.memory_table_state.address = 0;
            },
            _ => {}
        }
    }

    fn handle_output_input(&mut self, action: KeyPress, mode: Mode) {
    }

    fn handle_state_input(&mut self, action: KeyPress, mode: Mode) {
    }

    fn render_display_textline(&self, frame: &mut Frame<'_>) {
        if self.display_textline.is_none() { return; }
        let text = self.display_textline.as_ref().unwrap();
        let area = frame.area();
        let disp_layout = Layout::vertical(vec![Constraint::Percentage(0), Constraint::Length(1)]);
        let displine_widget = Line::raw(text)
            .on_black();
        frame.render_widget(Clear, disp_layout.areas::<2>(area)[1]);
        frame.render_widget(displine_widget, disp_layout.areas::<2>(area)[1]);
    }

    fn handle_command(&mut self, command: String) -> bool {
        match command.as_str() {
            "q" => {
                return true;
            }
            _ => {}
        }
        match self.current_window {
            WindowSelection::Memory => {
                if let Ok(address) = u16::from_str_radix(&command, 16) {
                    self.memory_table_state.address = address;
                }
            }
            _ => {}
        }

        false
    }
}
