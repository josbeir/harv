use crate::action::{Action, FormMode};
use crate::theme::Theme;
use harv_core::{ProjectAssignment, TaskAssignment};
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListState, Paragraph};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    ProjectList,
    TaskList,
    Date,
    Hours,
    Notes,
}

pub struct TimeEntryForm {
    entry_id: Option<u64>,
    last_project_id: Option<u64>,
    last_task_id: Option<u64>,
    mode: FormMode,
    is_running: bool,
    assignments: Vec<ProjectAssignment>,
    filtered_assignments: Vec<usize>,
    tasks: Vec<TaskAssignment>,
    filtered_tasks: Vec<usize>,
    project_list: ListState,
    task_list: ListState,
    selected_project_id: Option<u64>,
    project_search: String,
    task_search: String,
    date: String,
    hours: String,
    notes: String,
    cursor_pos: usize,
    active: Field,
    visible: bool,
    tasks_loading: bool,
    assignments_loading: bool,
}

impl TimeEntryForm {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        last_project_id: Option<u64>,
        last_task_id: Option<u64>,
        project_name: Option<String>,
        mode: FormMode,
        entry_id: Option<u64>,
        entry_date: Option<String>,
        entry_hours: Option<String>,
        entry_notes: Option<String>,
        is_running: bool,
    ) -> Self {
        let date = entry_date
            .unwrap_or_else(|| harv_core::datetime::format_date(harv_core::datetime::today()));

        let mut project_list = ListState::default();
        if last_project_id.is_some() {
            project_list.select(Some(0));
        }

        Self {
            entry_id,
            last_project_id,
            last_task_id,
            mode,
            is_running,
            assignments: Vec::new(),
            filtered_assignments: Vec::new(),
            tasks: Vec::new(),
            filtered_tasks: Vec::new(),
            project_list,
            task_list: ListState::default(),
            selected_project_id: last_project_id,
            project_search: project_name.unwrap_or_default(),
            task_search: String::new(),
            date,
            hours: entry_hours.unwrap_or_default(),
            notes: entry_notes.unwrap_or_default(),
            cursor_pos: 0,
            active: Field::ProjectList,
            visible: true,
            tasks_loading: false,
            assignments_loading: true,
        }
    }

    pub fn update_assignments(&mut self, assignments: Vec<ProjectAssignment>) {
        self.assignments = assignments;
        self.filter_projects();

        #[allow(clippy::collapsible_if)]
        if let Some(pid) = self.last_project_id {
            if let Some(pos) = self
                .filtered_assignments
                .iter()
                .position(|&i| self.assignments[i].project.id == pid)
            {
                self.project_list.select(Some(pos));
                self.selected_project_id = Some(pid);
                self.load_tasks_for_selected();
            }
        }
        self.assignments_loading = false;
    }

    pub fn update_tasks(&mut self, tasks: Vec<TaskAssignment>) {
        self.tasks = tasks;
        self.tasks_loading = false;
        self.task_search.clear();
        self.filter_tasks();
        if let Some(tid) = self.last_task_id {
            if let Some(pos) = self
                .filtered_tasks
                .iter()
                .position(|&i| self.tasks[i].task.id == tid)
            {
                self.task_list.select(Some(pos));
            } else {
                self.task_list.select(if self.filtered_tasks.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
        } else {
            self.task_list.select(if self.filtered_tasks.is_empty() {
                None
            } else {
                Some(0)
            });
        }
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn set_date(&mut self, d: String) {
        self.date = d;
    }

    fn filter_projects(&mut self) {
        let q = self.project_search.to_lowercase();
        self.filtered_assignments = if q.is_empty() {
            (0..self.assignments.len()).collect()
        } else {
            let mut scored: Vec<(usize, i32)> = self
                .assignments
                .iter()
                .enumerate()
                .filter_map(|(i, a)| {
                    let name_score = harv_core::text::fuzzy_score(&q, &a.project.name);
                    let code_score = a.project_code.as_deref().and_then(|c| {
                        let s = harv_core::text::fuzzy_score(&q, c);
                        if s >= 0 { Some(s) } else { None }
                    });
                    let client_score = a.client.as_ref().and_then(|c| {
                        let s = harv_core::text::fuzzy_score(&q, &c.name);
                        if s >= 0 { Some(s) } else { None }
                    });
                    let score = name_score
                        .max(code_score.unwrap_or(-1))
                        .max(client_score.unwrap_or(-1));
                    if score >= 0 { Some((i, score)) } else { None }
                })
                .collect();
            scored.sort_by_key(|(_, s)| -s);
            scored.into_iter().map(|(i, _)| i).collect()
        };
    }

    fn filter_tasks(&mut self) {
        let q = self.task_search.to_lowercase();
        self.filtered_tasks = if q.is_empty() {
            (0..self.tasks.len()).collect()
        } else {
            let mut scored: Vec<(usize, i32)> = self
                .tasks
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    let score = harv_core::text::fuzzy_score(&q, &t.task.name);
                    if score >= 0 { Some((i, score)) } else { None }
                })
                .collect();
            scored.sort_by_key(|(_, s)| -s);
            scored.into_iter().map(|(i, _)| i).collect()
        };
    }

    fn selected_project(&self) -> Option<&ProjectAssignment> {
        self.project_list
            .selected()
            .and_then(|i| self.filtered_assignments.get(i))
            .and_then(|&i| self.assignments.get(i))
    }

    fn selected_task(&self) -> Option<&TaskAssignment> {
        self.task_list
            .selected()
            .and_then(|i| self.filtered_tasks.get(i))
            .and_then(|&i| self.tasks.get(i))
    }

    fn is_full_layout(&self) -> bool {
        self.mode != FormMode::Start && !self.is_running
    }

    fn submit_entry(&mut self) -> Vec<Action> {
        let project_id = self.selected_project().map(|a| a.project.id);
        let task_id = self.selected_task().map(|t| t.task.id);

        let (pid, tid) = match (project_id, task_id) {
            (Some(p), Some(t)) => (p, t),
            _ => return vec![],
        };

        let (hours, notes) = if self.is_full_layout() {
            let h = if self.hours.trim().is_empty() {
                None
            } else {
                harv_core::datetime::parse_hours(self.hours.trim()).ok()
            };
            let n = if self.notes.is_empty() {
                None
            } else {
                Some(self.notes.clone())
            };
            (h, n)
        } else {
            let n = if self.notes.is_empty() {
                None
            } else {
                Some(self.notes.clone())
            };
            (None, n)
        };

        self.visible = false;
        if self.mode == FormMode::Edit {
            if let Some(eid) = self.entry_id {
                vec![
                    Action::EditEntry {
                        entry_id: eid,
                        project_id: pid,
                        task_id: tid,
                        spent_date: self.date.clone(),
                        hours,
                        notes,
                    },
                    Action::SwitchView(crate::action::ViewId::Dashboard),
                ]
            } else {
                vec![Action::SwitchView(crate::action::ViewId::Dashboard)]
            }
        } else {
            vec![
                Action::CreateEntry {
                    project_id: pid,
                    task_id: tid,
                    spent_date: self.date.clone(),
                    hours,
                    notes,
                },
                Action::SwitchView(crate::action::ViewId::Dashboard),
            ]
        }
    }

    fn select_project_inner(&mut self) {
        let pid = self.selected_project().map(|a| a.project.id);
        self.selected_project_id = pid;
        self.load_tasks_for_selected();
    }

    fn load_tasks_for_selected(&mut self) {
        self.tasks_loading = true;
        self.tasks.clear();
        self.task_search.clear();
        let tasks = self
            .selected_project()
            .map(|pa| pa.task_assignments.clone())
            .unwrap_or_default();
        self.update_tasks(tasks);
    }

    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme, tick: u64) {
        if !self.visible {
            return;
        }

        let popup = if self.is_full_layout() {
            crate::popup::centered_rect_fixed(area.width.saturating_sub(6).min(72), 22, area)
        } else {
            crate::popup::centered_rect_fixed(area.width.saturating_sub(6).min(60), 17, area)
        };
        f.render_widget(Clear, popup);

        let title = match self.mode {
            FormMode::Start => format!(" {} ", harv_core::t("tui-form-title-start")),
            FormMode::Create => format!(" {} ", harv_core::t("tui-form-title-create")),
            FormMode::Edit => format!(" {} ", harv_core::t("tui-form-title-edit")),
        };

        let block = Block::new()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.primary))
            .style(Style::new().bg(theme.surface));

        let inner = block.inner(popup);
        f.render_widget(block, popup);

        let inner_width = inner.width.saturating_sub(2);
        let content_x = inner.x + 1;

        if self.is_full_layout() {
            let layout = Layout::vertical([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(inner);

            self.render_project_section(layout[0], inner_width, content_x, f, theme, tick);
            self.render_task_section(layout[1], inner_width, content_x, f, theme, tick);
            self.render_text_field(
                &harv_core::t("tui-form-date-label"),
                &self.date,
                self.cursor_pos,
                self.active == Field::Date,
                Rect {
                    x: content_x,
                    y: layout[2].y,
                    width: inner_width,
                    height: layout[2].height,
                },
                f,
                theme,
            );
            self.render_text_field(
                &harv_core::t("tui-form-hours-label"),
                &self.hours,
                self.cursor_pos,
                self.active == Field::Hours,
                Rect {
                    x: content_x,
                    y: layout[3].y,
                    width: inner_width,
                    height: layout[3].height,
                },
                f,
                theme,
            );
            self.render_text_field(
                &harv_core::t("tui-form-notes-label"),
                &self.notes,
                self.cursor_pos,
                self.active == Field::Notes,
                Rect {
                    x: content_x,
                    y: layout[4].y,
                    width: inner_width,
                    height: layout[4].height,
                },
                f,
                theme,
            );

            let help = Span::styled(
                format!(" {} ", harv_core::t("tui-form-help-create")),
                Style::new().fg(theme.muted),
            );
            f.render_widget(Paragraph::new(help), layout[5]);
        } else {
            let layout = Layout::vertical([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);

            self.render_project_section(layout[0], inner_width, content_x, f, theme, tick);
            self.render_task_section(layout[1], inner_width, content_x, f, theme, tick);
            self.render_text_field(
                &harv_core::t("tui-form-notes-label"),
                &self.notes,
                self.cursor_pos,
                self.active == Field::Notes,
                Rect {
                    x: content_x,
                    y: layout[2].y,
                    width: inner_width,
                    height: layout[2].height,
                },
                f,
                theme,
            );

            let help_text = match self.mode {
                FormMode::Start => format!(" {} ", harv_core::t("tui-form-help-start")),
                FormMode::Edit => format!(" {} ", harv_core::t("tui-form-help-edit")),
                FormMode::Create => unreachable!(),
            };
            let help = Span::styled(help_text, Style::new().fg(theme.muted));
            f.render_widget(Paragraph::new(help), layout[4]);
        }
    }

    fn render_project_section(
        &mut self,
        area: Rect,
        width: u16,
        x: u16,
        f: &mut Frame,
        theme: &Theme,
        tick: u64,
    ) {
        let active = self.active == Field::ProjectList;
        let border_style = if active {
            Style::new().fg(theme.primary)
        } else {
            Style::new().fg(theme.border)
        };

        let title = if !self.project_search.is_empty() {
            harv_core::t_args(
                "tui-form-project-search",
                &[("search", self.project_search.clone())],
            )
        } else {
            format!(" {} ", harv_core::t("tui-form-project-title"))
        };

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let items: Vec<Line> = if self.assignments_loading {
            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let text = format!(
                "{} {}",
                spinner[(tick as usize) % spinner.len()],
                harv_core::t("tui-form-project-loading")
            );
            vec![Line::from(Span::styled(text, Style::new().fg(theme.muted)))]
        } else if self.assignments.is_empty() {
            vec![Line::from(Span::styled(
                harv_core::t("tui-form-project-empty"),
                Style::new().fg(theme.muted),
            ))]
        } else {
            self.filtered_assignments
                .iter()
                .map(|&i| {
                    let a = &self.assignments[i];
                    let proj_display = harv_core::text::format_project_display(
                        &a.project.name,
                        a.project_code.as_deref(),
                    );
                    let text = match &a.client {
                        Some(client) => format!("{} ({})", proj_display, client.name),
                        None => proj_display,
                    };
                    if active {
                        Line::from(Span::styled(text, Style::new().fg(theme.fg)))
                    } else {
                        Line::from(Span::styled(text, Style::new().fg(theme.muted)))
                    }
                })
                .collect()
        };

        let list_area = Rect {
            x,
            y: area.y,
            width,
            height: area.height,
        };
        let list = List::new(items).block(block).highlight_style(
            Style::new()
                .fg(theme.highlight)
                .bg(theme.surface)
                .add_modifier(Modifier::BOLD),
        );

        f.render_stateful_widget(list, list_area, &mut self.project_list);
    }

    fn render_task_section(
        &mut self,
        area: Rect,
        width: u16,
        x: u16,
        f: &mut Frame,
        theme: &Theme,
        _tick: u64,
    ) {
        let active = self.active == Field::TaskList;
        let border_style = if active {
            Style::new().fg(theme.primary)
        } else {
            Style::new().fg(theme.border)
        };

        let title = if !self.task_search.is_empty() {
            harv_core::t_args(
                "tui-form-task-search",
                &[("search", self.task_search.clone())],
            )
        } else {
            format!(" {} ", harv_core::t("tui-form-task-title"))
        };

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let list_text: Vec<String> = if self.assignments_loading {
            vec![harv_core::t("tui-app-loading-generic")]
        } else if self.tasks.is_empty() && self.selected_project().is_some() {
            vec![harv_core::t("tui-form-task-empty")]
        } else if self.selected_project().is_none() {
            vec![harv_core::t("tui-form-task-select-first")]
        } else {
            self.filtered_tasks
                .iter()
                .map(|&i| {
                    let t = &self.tasks[i];
                    let rate = t
                        .hourly_rate
                        .map_or("$0.00/h".into(), |r| format!("${:.2}/h", r));
                    format!("{}  {}", t.task.name, rate)
                })
                .collect()
        };

        let items: Vec<Line> = list_text
            .iter()
            .map(|t| {
                if active {
                    Line::from(Span::styled(t.clone(), Style::new().fg(theme.fg)))
                } else {
                    Line::from(Span::styled(t.clone(), Style::new().fg(theme.muted)))
                }
            })
            .collect();

        let list_area = Rect {
            x,
            y: area.y,
            width,
            height: area.height,
        };
        let list = List::new(items).block(block).highlight_style(
            Style::new()
                .fg(theme.highlight)
                .bg(theme.surface)
                .add_modifier(Modifier::BOLD),
        );

        f.render_stateful_widget(list, list_area, &mut self.task_list);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_text_field(
        &self,
        label: &str,
        value: &str,
        cursor_pos: usize,
        active: bool,
        area: Rect,
        f: &mut Frame,
        theme: &Theme,
    ) {
        let border_style = if active {
            Style::new().fg(theme.primary)
        } else {
            Style::new().fg(theme.border)
        };

        let block = Block::new()
            .title(label)
            .borders(Borders::ALL)
            .border_style(border_style);

        let display = if value.is_empty() {
            Span::styled(
                harv_core::t("tui-form-empty-field"),
                Style::new().fg(theme.muted),
            )
        } else {
            Span::styled(value.to_string(), Style::new().fg(theme.fg))
        };

        let inner = block.inner(area);
        f.render_widget(&block, area);
        f.render_widget(Paragraph::new(display.clone()), inner);

        if active {
            let cursor_x = inner.x + cursor_pos.min(inner.width.saturating_sub(1) as usize) as u16;
            f.set_cursor_position((cursor_x.min(inner.right().saturating_sub(1)), inner.y));
        }
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match self.active {
            Field::ProjectList => self.handle_project_key(key),
            Field::TaskList => self.handle_task_key(key),
            Field::Date | Field::Hours | Field::Notes => self.handle_text_key(key),
        }
    }

    fn handle_project_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
                vec![Action::SwitchView(crate::action::ViewId::Dashboard)]
            }
            KeyCode::Tab => {
                self.active = Field::TaskList;
                vec![]
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let max = self.filtered_assignments.len().saturating_sub(1);
                let i = self.project_list.selected().map_or(0, |i| (i + 1).min(max));
                self.project_list.select(Some(i));
                self.select_project_inner();
                vec![]
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self
                    .project_list
                    .selected()
                    .map_or(0, |i| i.saturating_sub(1));
                self.project_list.select(Some(i));
                self.select_project_inner();
                vec![]
            }
            KeyCode::Enter => {
                self.select_project_inner();
                self.active = Field::TaskList;
                vec![]
            }
            KeyCode::Backspace => {
                self.project_search.pop();
                self.filter_projects();
                self.project_list.select(Some(0));
                vec![]
            }
            KeyCode::Char(c) => {
                self.project_search.push(c);
                self.filter_projects();
                self.project_list.select(Some(0));
                vec![]
            }
            _ => vec![],
        }
    }

    fn handle_task_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
                vec![Action::SwitchView(crate::action::ViewId::Dashboard)]
            }
            KeyCode::Tab => {
                self.active = if self.is_full_layout() {
                    Field::Date
                } else {
                    Field::Notes
                };
                self.cursor_pos = self.active_text_field().len();
                vec![]
            }
            KeyCode::BackTab => {
                self.active = Field::ProjectList;
                vec![]
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let max = self.filtered_tasks.len().saturating_sub(1);
                let i = self.task_list.selected().map_or(0, |i| (i + 1).min(max));
                self.task_list.select(Some(i));
                vec![]
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self.task_list.selected().map_or(0, |i| i.saturating_sub(1));
                self.task_list.select(Some(i));
                vec![]
            }
            KeyCode::Enter => {
                self.active = if self.is_full_layout() {
                    Field::Date
                } else {
                    Field::Notes
                };
                self.cursor_pos = self.active_text_field().len();
                vec![]
            }
            KeyCode::Backspace => {
                self.task_search.pop();
                self.filter_tasks();
                self.task_list.select(if self.filtered_tasks.is_empty() {
                    None
                } else {
                    Some(0)
                });
                vec![]
            }
            KeyCode::Char(c) => {
                self.task_search.push(c);
                self.filter_tasks();
                self.task_list.select(if self.filtered_tasks.is_empty() {
                    None
                } else {
                    Some(0)
                });
                vec![]
            }
            _ => {
                vec![]
            }
        }
    }

    fn handle_text_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
                vec![Action::SwitchView(crate::action::ViewId::Dashboard)]
            }
            KeyCode::Tab => {
                self.active = match self.active {
                    Field::Date => Field::Hours,
                    Field::Hours => Field::Notes,
                    Field::Notes => Field::ProjectList,
                    _ => Field::ProjectList,
                };
                if Self::is_text_field(self.active) {
                    self.cursor_pos = self.active_text_field().len();
                }
                vec![]
            }
            KeyCode::BackTab => {
                self.active = match self.active {
                    Field::Date => Field::TaskList,
                    Field::Hours => Field::Date,
                    Field::Notes => {
                        if self.is_full_layout() {
                            Field::Hours
                        } else {
                            Field::TaskList
                        }
                    }
                    _ => Field::ProjectList,
                };
                if Self::is_text_field(self.active) {
                    self.cursor_pos = self.active_text_field().len();
                }
                vec![]
            }
            KeyCode::Char('g') if self.active == Field::Date => {
                vec![Action::OpenDatePicker]
            }
            KeyCode::Enter => {
                match self.active {
                    Field::Date => self.active = Field::Hours,
                    Field::Hours => self.active = Field::Notes,
                    _ => return self.submit_entry(),
                };
                self.cursor_pos = self.active_text_field().len();
                vec![]
            }
            KeyCode::Left => {
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
                vec![]
            }
            KeyCode::Right => {
                let max = self.active_text_field().len();
                if self.cursor_pos < max {
                    self.cursor_pos += 1;
                }
                vec![]
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                vec![]
            }
            KeyCode::End => {
                self.cursor_pos = self.active_text_field().len();
                vec![]
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    let pos = self.cursor_pos;
                    self.active_text_field_mut().remove(pos);
                }
                vec![]
            }
            KeyCode::Delete => {
                let pos = self.cursor_pos;
                let s = self.active_text_field_mut();
                if pos < s.len() {
                    s.remove(pos);
                }
                vec![]
            }
            KeyCode::Char(c) => {
                let pos = self.cursor_pos;
                self.active_text_field_mut().insert(pos, c);
                self.cursor_pos = pos + 1;
                vec![]
            }
            _ => vec![],
        }
    }

    fn is_text_field(active: Field) -> bool {
        matches!(active, Field::Date | Field::Hours | Field::Notes)
    }

    fn active_text_field(&self) -> &str {
        match self.active {
            Field::Date => &self.date,
            Field::Hours => &self.hours,
            Field::Notes => &self.notes,
            _ => unreachable!(),
        }
    }

    fn active_text_field_mut(&mut self) -> &mut String {
        match self.active {
            Field::Date => &mut self.date,
            Field::Hours => &mut self.hours,
            Field::Notes => &mut self.notes,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harv_core::{ProjectAssignment, Reference, TaskAssignment};

    fn project_assignment(id: u64, name: &str) -> ProjectAssignment {
        ProjectAssignment {
            id,
            project: Reference {
                id,
                name: name.into(),
            },
            project_code: None,
            client: None,
            task_assignments: vec![],
            is_active: true,
        }
    }

    fn task_assignment(id: u64, name: &str) -> TaskAssignment {
        TaskAssignment {
            id,
            task: Reference {
                id,
                name: name.into(),
            },
            billable: true,
            hourly_rate: Some(75.0),
            is_active: true,
            budget: None,
        }
    }

    fn key_press(code: KeyCode) -> ratatui::crossterm::event::KeyEvent {
        ratatui::crossterm::event::KeyEvent::new(
            code,
            ratatui::crossterm::event::KeyModifiers::NONE,
        )
    }

    #[test]
    fn test_new_start_form() {
        let f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        assert!(matches!(f.mode, FormMode::Start));
        assert!(f.selected_project_id.is_none());
        assert!(f.hours.is_empty());
        assert!(f.notes.is_empty());
    }

    #[test]
    fn test_new_create_form() {
        let f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        assert!(matches!(f.mode, FormMode::Create));
    }

    #[test]
    fn test_new_edit_form() {
        let f = TimeEntryForm::new(
            Some(10),
            Some(20),
            None,
            FormMode::Edit,
            Some(42),
            Some("2026-06-09".into()),
            Some("1.5".into()),
            Some("my notes".into()),
            false,
        );
        assert!(matches!(f.mode, FormMode::Edit));
        assert_eq!(f.entry_id, Some(42));
        assert_eq!(f.date, "2026-06-09");
        assert_eq!(f.hours, "1.5");
        assert_eq!(f.notes, "my notes");
        assert_eq!(f.last_project_id, Some(10));
        assert_eq!(f.last_task_id, Some(20));
    }

    #[test]
    fn test_update_assignments_pre_selects_project() {
        let mut f = TimeEntryForm::new(
            Some(10),
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        let assignments = vec![
            project_assignment(5, "Alpha"),
            project_assignment(10, "Beta"),
        ];
        f.update_assignments(assignments);
        assert_eq!(f.selected_project_id, Some(10));
    }

    #[test]
    fn test_update_assignments_no_match() {
        let mut f = TimeEntryForm::new(
            Some(99),
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        let assignments = vec![project_assignment(5, "Alpha")];
        f.update_assignments(assignments);
        assert_eq!(f.project_list.selected(), Some(0)); // unchanged
    }

    #[test]
    fn test_assignments_loading_initially_true() {
        let f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        assert!(f.assignments_loading);
    }

    #[test]
    fn test_assignments_loading_false_after_update() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        f.update_assignments(vec![project_assignment(5, "Alpha")]);
        assert!(!f.assignments_loading);
    }

    #[test]
    fn test_assignments_loading_false_after_empty_update() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        f.update_assignments(vec![]);
        assert!(!f.assignments_loading);
    }

    #[test]
    fn test_update_tasks_pre_selects_task() {
        let mut f = TimeEntryForm::new(
            Some(10),
            Some(30),
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        f.selected_project_id = Some(10);
        let tasks = vec![
            task_assignment(20, "Design"),
            task_assignment(30, "Development"),
        ];
        f.update_tasks(tasks);
        assert!(!f.tasks_loading);
        assert_eq!(f.task_list.selected(), Some(1)); // task 30 at index 1
    }

    #[test]
    fn test_update_tasks_no_match_selects_first() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        f.selected_project_id = Some(10);
        let tasks = vec![task_assignment(20, "Design")];
        f.update_tasks(tasks);
        assert_eq!(f.task_list.selected(), Some(0));
    }

    #[test]
    fn test_project_search_filters() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        f.assignments = vec![
            project_assignment(1, "Alpha"),
            project_assignment(2, "Beta"),
            project_assignment(3, "Alphabet"),
        ];
        f.filter_projects();
        assert_eq!(f.filtered_assignments.len(), 3);

        f.project_search = "bet".into();
        f.filter_projects();
        assert_eq!(f.filtered_assignments.len(), 2); // Beta and Alphabet
    }

    #[test]
    fn test_tab_cycles_fields_in_start_mode() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        assert_eq!(f.active, Field::ProjectList);

        // ProjectList Tab -> TaskList
        let _ = f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::TaskList);

        // TaskList Tab -> Notes
        let _ = f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::Notes);

        // Notes Tab -> ProjectList
        let _ = f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::ProjectList);

        // Notes BackTab -> TaskList
        f.active = Field::Notes;
        let _ = f.handle_key(&key_press(KeyCode::BackTab));
        assert_eq!(f.active, Field::TaskList);
    }

    #[test]
    fn test_tab_cycles_fields_in_create_mode() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        assert_eq!(f.active, Field::ProjectList);

        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::TaskList);

        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::Date);

        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::Hours);

        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::Notes);

        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::ProjectList);
    }

    #[test]
    fn test_esc_cancels_form() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        assert!(f.visible);
        let actions = f.handle_key(&key_press(KeyCode::Esc));
        assert!(!f.visible);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_submit_requires_project_and_task() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        let actions = f.submit_entry();
        assert!(actions.is_empty()); // no project/task selected
    }

    #[test]
    fn test_submit_with_project_and_task() {
        let mut f = TimeEntryForm::new(
            Some(10),
            Some(20),
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        f.assignments = vec![project_assignment(10, "Beta")];
        f.filtered_assignments = vec![0];
        f.project_list.select(Some(0));
        f.selected_project_id = Some(10);

        f.tasks = vec![task_assignment(20, "Development")];
        f.filter_tasks();
        f.task_list.select(Some(0));

        let actions = f.submit_entry();
        assert_eq!(actions.len(), 2);
        assert!(matches!(
            actions[0],
            Action::CreateEntry {
                project_id: 10,
                task_id: 20,
                hours: None,
                notes: None,
                ..
            }
        ));
    }

    #[test]
    fn test_submit_edit_mode_dispatches_edit() {
        let mut f = TimeEntryForm::new(
            Some(10),
            Some(20),
            None,
            FormMode::Edit,
            Some(42),
            Some("2026-06-09".into()),
            Some("1.5".into()),
            None,
            false,
        );
        f.assignments = vec![project_assignment(10, "Beta")];
        f.filtered_assignments = vec![0];
        f.project_list.select(Some(0));
        f.selected_project_id = Some(10);
        f.tasks = vec![task_assignment(20, "Development")];
        f.filter_tasks();
        f.task_list.select(Some(0));

        let actions = f.submit_entry();
        assert!(matches!(actions[0], Action::EditEntry { entry_id: 42, .. }));
    }

    #[test]
    fn test_submit_start_mode_no_hours() {
        let mut f = TimeEntryForm::new(
            Some(10),
            Some(20),
            None,
            FormMode::Start,
            None,
            None,
            None,
            Some("my notes".into()),
            false,
        );
        f.assignments = vec![project_assignment(10, "Beta")];
        f.filtered_assignments = vec![0];
        f.project_list.select(Some(0));
        f.selected_project_id = Some(10);
        f.tasks = vec![task_assignment(20, "Development")];
        f.filter_tasks();
        f.task_list.select(Some(0));

        let actions = f.submit_entry();
        assert!(matches!(
            actions[0],
            Action::CreateEntry {
                hours: None,
                notes: Some(ref n),
                ..
            } if n == "my notes"
        ));
    }

    #[test]
    fn test_task_search_filters() {
        let mut f = TimeEntryForm::new(
            Some(10),
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        f.selected_project_id = Some(10);
        f.update_tasks(vec![
            task_assignment(20, "Development"),
            task_assignment(30, "Design"),
            task_assignment(40, "Deployment"),
        ]);

        // No filter — all three
        assert_eq!(f.filtered_tasks.len(), 3);

        // Filter by "dev" — should match "Development" only
        f.task_search = "dev".into();
        f.filter_tasks();
        assert_eq!(f.filtered_tasks.len(), 1);
        assert_eq!(f.tasks[f.filtered_tasks[0]].task.id, 20);

        // Filter by "de" — matches all three (Development, Design, Deployment)
        f.task_search = "de".into();
        f.filter_tasks();
        assert_eq!(f.filtered_tasks.len(), 3);

        // Filter by "xyz" — no match
        f.task_search = "xyz".into();
        f.filter_tasks();
        assert!(f.filtered_tasks.is_empty());

        // Backspace clears
        f.task_search = "".into();
        f.filter_tasks();
        assert_eq!(f.filtered_tasks.len(), 3);
    }

    #[test]
    fn test_is_full_layout() {
        // Start mode: always restricted (no date/hours)
        let form = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Start,
            None,
            None,
            None,
            None,
            false,
        );
        assert!(!form.is_full_layout());

        // Create mode: always full layout
        let form = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        assert!(form.is_full_layout());

        // Edit + stopped: full layout
        let form = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Edit,
            Some(1),
            None,
            None,
            None,
            false,
        );
        assert!(form.is_full_layout());

        // Edit + running: restricted layout
        let form = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Edit,
            Some(1),
            None,
            None,
            None,
            true,
        );
        assert!(!form.is_full_layout());
    }

    #[test]
    fn test_submit_edit_running_dispatches_edit() {
        let mut f = TimeEntryForm::new(
            Some(10),
            Some(20),
            None,
            FormMode::Edit,
            Some(42),
            Some("2026-06-09".into()),
            Some("1.5".into()),
            Some("my notes".into()),
            true,
        );
        f.assignments = vec![project_assignment(10, "Beta")];
        f.filtered_assignments = vec![0];
        f.project_list.select(Some(0));
        f.selected_project_id = Some(10);
        f.tasks = vec![task_assignment(20, "Development")];
        f.filter_tasks();
        f.task_list.select(Some(0));

        let actions = f.submit_entry();
        assert!(matches!(
            actions[0],
            Action::EditEntry {
                entry_id: 42,
                project_id: 10,
                task_id: 20,
                hours: None,
                ..
            }
        ));
        // Notes still passed through
        assert!(matches!(
            actions[0],
            Action::EditEntry {
                notes: Some(ref n),
                ..
            } if n == "my notes"
        ));
    }

    #[test]
    fn test_tab_cycles_fields_in_running_edit_mode() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Edit,
            Some(1),
            None,
            None,
            None,
            true,
        );
        assert_eq!(f.active, Field::ProjectList);

        // ProjectList Tab → TaskList
        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::TaskList);

        // TaskList Tab → Notes (no Date/Hours when running)
        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::Notes);

        // Notes Tab → ProjectList
        f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::ProjectList);

        // Notes BackTab → TaskList
        f.active = Field::Notes;
        f.handle_key(&key_press(KeyCode::BackTab));
        assert_eq!(f.active, Field::TaskList);
    }

    #[test]
    fn test_task_enter_advances_to_notes_when_running() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Edit,
            Some(1),
            None,
            None,
            None,
            true,
        );
        f.active = Field::TaskList;
        f.handle_key(&key_press(KeyCode::Enter));
        assert_eq!(f.active, Field::Notes);
    }

    #[test]
    fn test_notes_enter_submits_when_running() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Edit,
            Some(1),
            None,
            None,
            None,
            true,
        );
        f.active = Field::Notes;
        let actions = f.handle_key(&key_press(KeyCode::Enter));
        // Submit requires project + task, so returns empty
        // but it did try to submit rather than advance to another field
        assert!(actions.is_empty());
    }

    #[test]
    fn test_text_enter_on_date_advances_to_hours_in_full_layout() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        f.active = Field::Date;
        f.handle_key(&key_press(KeyCode::Enter));
        assert_eq!(f.active, Field::Hours);
    }

    #[test]
    fn test_hours_enter_advances_to_notes_in_full_layout() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        f.active = Field::Hours;
        f.handle_key(&key_press(KeyCode::Enter));
        assert_eq!(f.active, Field::Notes);
    }

    #[test]
    fn test_g_in_date_field_opens_picker() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        f.active = Field::Date;
        let actions = f.handle_key(&key_press(KeyCode::Char('g')));
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::OpenDatePicker));
    }

    #[test]
    fn test_set_date_updates_field() {
        let mut f = TimeEntryForm::new(
            None,
            None,
            None,
            FormMode::Create,
            None,
            None,
            None,
            None,
            false,
        );
        f.set_date("2025-01-15".into());
        assert_eq!(f.date(), "2025-01-15");
    }
}
