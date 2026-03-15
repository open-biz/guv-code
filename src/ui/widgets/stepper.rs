use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use crate::agent_logic::AgentPhase;
use crate::ui::theme;

#[derive(Debug, Clone)]
pub struct StepState {
    pub label: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

pub struct AgentStepper<'a> {
    steps: &'a [StepState],
    tick: usize,
    block: Option<Block<'a>>,
}

impl<'a> AgentStepper<'a> {
    pub fn new(steps: &'a [StepState]) -> Self {
        Self {
            steps,
            tick: 0,
            block: None,
        }
    }

    pub fn tick(mut self, tick: usize) -> Self {
        self.tick = tick;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn spinner_frame(&self) -> &'static str {
        theme::ICON_SPINNER_FRAMES[(self.tick / 2) % theme::ICON_SPINNER_FRAMES.len()]
    }
}

impl<'a> Widget for AgentStepper<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        for (i, step) in self.steps.iter().enumerate() {
            let (icon, icon_style) = match step.status {
                StepStatus::Completed => (
                    theme::ICON_CHECK.to_string(),
                    Style::default().fg(theme::GREEN),
                ),
                StepStatus::Active => (
                    self.spinner_frame().to_string(),
                    Style::default().fg(theme::ACCENT_SECONDARY),
                ),
                StepStatus::Failed => (
                    theme::ICON_CROSS.to_string(),
                    Style::default().fg(theme::RED),
                ),
                StepStatus::Pending => (
                    theme::ICON_RADIO_OFF.to_string(),
                    Style::default().fg(theme::FG_SUBTLE),
                ),
            };

            let label_style = match step.status {
                StepStatus::Active => Style::default()
                    .fg(theme::FG_BASE)
                    .add_modifier(Modifier::BOLD),
                StepStatus::Completed => Style::default().fg(theme::FG_MUTED),
                StepStatus::Failed => Style::default().fg(theme::RED),
                StepStatus::Pending => Style::default().fg(theme::FG_SUBTLE),
            };

            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(icon, icon_style),
                Span::raw(" "),
                Span::styled(&step.label, label_style),
            ]));

            // Connector line between steps (not after last)
            if i < self.steps.len() - 1 {
                let connector_style = if step.status == StepStatus::Completed {
                    Style::default().fg(theme::GREEN_DARK)
                } else {
                    Style::default().fg(theme::FG_DIM)
                };
                lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("│", connector_style),
                ]));
            }
        }

        let para = Paragraph::new(lines).wrap(Wrap { trim: false });
        para.render(inner, buf);
    }
}

/// Build step states from the current agent phase
pub fn steps_from_phase(phase: &AgentPhase) -> Vec<StepState> {
    let phases = [
        ("Mapping Codebase", AgentPhase::Mapping),
        ("Planning Edits", AgentPhase::Planning),
        ("Executing Changes", AgentPhase::Coding),
        ("Reviewing Output", AgentPhase::Reviewing),
    ];

    let current_idx = match phase {
        AgentPhase::Idle => usize::MAX,
        AgentPhase::Mapping => 0,
        AgentPhase::Planning => 1,
        AgentPhase::Coding => 2,
        AgentPhase::Reviewing => 3,
        AgentPhase::Complete => 4,
    };

    phases
        .iter()
        .enumerate()
        .map(|(i, (label, _))| {
            let status = if current_idx == usize::MAX {
                StepStatus::Pending
            } else if i < current_idx {
                StepStatus::Completed
            } else if i == current_idx {
                StepStatus::Active
            } else {
                StepStatus::Pending
            };
            StepState {
                label: label.to_string(),
                status,
            }
        })
        .collect()
}
