use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::text::ToSpan;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::{DefaultTerminal, Frame, text::Line};

use crate::LC3Simulator;

#[derive(PartialEq)]
pub enum WindowSelection {
    Memory,
    Output,
    State
}

pub struct Debugger {
    sim: LC3Simulator,
    current_window: WindowSelection
}

impl Default for Debugger {
    fn default() -> Self {
        Self { sim: LC3Simulator::with_os(), current_window: WindowSelection::Output }
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
                t.draw(|f: &mut Frame| {self.render_frame(f)})?;
                let event = crossterm::event::read()?;
                let should_exit = self.handle_event(event);
                if should_exit {
                    break Ok(())
                }
            }
        })
    }

    pub fn handle_event(&mut self, event: crossterm::event::Event) -> bool {
        match event {
            crossterm::event::Event::Key(key_event) => {
                return self.handle_key_event(key_event);
            },
            _ => {}
        }
        false
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if key_event.code.is_esc() {return true;}

        false
    }

    pub fn render_frame(&mut self, frame: &mut Frame) {
        let title = Line::from("LC3 Debugger");
        frame.render_widget(title, frame.area());

        let [memory_area, right] = Layout::horizontal(Constraint::from_percentages([50, 50]))
            .areas(frame.area());

        let [state_area, output_area] = Layout::vertical(Constraint::from_percentages([50, 50]))
            .areas(right);

        self.render_memory(frame, memory_area);
        self.render_state(frame, state_area);
        self.render_output(frame, output_area);
    }

    fn draw_subwindow(&'_ self, title: String, selected: bool) -> Block<'_> {
        let mut title = Line::from(title);
        if selected {
            title = title.bold().underlined();
        }
        let block = Block::bordered()
            .title(title);

        block
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow("Output".to_string(), self.current_window == WindowSelection::Output);
        frame.render_widget(block, area);
    }                                                      
                                                           
    fn render_memory(&self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow("Memory".to_string(), self.current_window == WindowSelection::Memory);
        frame.render_widget(block, area);
    }

    fn render_state(&self, frame: &mut Frame, area: Rect) {
        let block = self.draw_subwindow("State".to_string(), self.current_window == WindowSelection::State);
        frame.render_widget(block, area);
    }
}
