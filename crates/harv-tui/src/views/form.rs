use crate::action::{Action, FormMode};
use crate::theme::Theme;
use harv_core::{ProjectAssignment, TaskAssignment};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListState, Paragraph};
use ratatui::Frame;

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
    assignments: Vec<ProjectAssignment>,
    filtered_assignments: Vec<usize>,
    tasks: Vec<TaskAssignment>,
    project_list: ListState,
    task_list: ListState,
    selected_project_id: Option<u64>,
    project_search: String,
    date: String,
    hours: String,
    notes: String,
    active: Field,
    visible: bool,
    loaded: bool,
    tasks_loading: bool,
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
            assignments: Vec::new(),
            filtered_assignments: Vec::new(),
            tasks: Vec::new(),
            project_list,
            task_list: ListState::default(),
            selected_project_id: last_project_id,
            project_search: project_name.unwrap_or_default(),
            date,
            hours: entry_hours.unwrap_or_default(),
            notes: entry_notes.unwrap_or_default(),
            active: Field::ProjectList,
            visible: true,
            loaded: false,
            tasks_loading: false,
        }
    }

    pub fn update_assignments(&mut self, assignments: Vec<ProjectAssignment>) -> Option<u64> {
        self.assignments = assignments;
        self.filter_projects();
        self.loaded = true;

        if let Some(pid) = self.last_project_id {
            if let Some(pos) = self
                .filtered_assignments
                .iter()
                .position(|&i| self.assignments[i].project.id == pid)
            {
                self.project_list.select(Some(pos));
                self.selected_project_id = Some(pid);
                self.tasks_loading = true;
                self.tasks.clear();
                return Some(pid);
            }
        }
        None
    }

    pub fn update_tasks(&mut self, tasks: Vec<TaskAssignment>) {
        self.tasks = tasks;
        self.tasks_loading = false;
        if let Some(tid) = self.last_task_id {
            if let Some(pos) = self.tasks.iter().position(|t| t.task.id == tid) {
                self.task_list.select(Some(pos));
            } else {
                self.task_list.select(Some(0));
            }
        } else {
            self.task_list.select(Some(0));
        }
    }

    fn filter_projects(&mut self) {
        let q = self.project_search.to_lowercase();
        self.filtered_assignments = if q.is_empty() {
            (0..self.assignments.len()).collect()
        } else {
            self.assignments
                .iter()
                .enumerate()
                .filter(|(_, a)| a.project.name.to_lowercase().contains(&q))
                .map(|(i, _)| i)
                .collect()
        };
    }

    fn selected_project(&self) -> Option<&ProjectAssignment> {
        self.project_list
            .selected()
            .and_then(|i| self.filtered_assignments.get(i))
            .and_then(|&i| self.assignments.get(i))
    }

    fn selected_task(&self) -> Option<&TaskAssignment> {
        self.task_list.selected().and_then(|i| self.tasks.get(i))
    }

    fn submit_entry(&mut self) -> Vec<Action> {
        let project_id = self.selected_project().map(|a| a.project.id);
        let task_id = self.selected_task().map(|t| t.task.id);

        let (pid, tid) = match (project_id, task_id) {
            (Some(p), Some(t)) => (p, t),
            _ => return vec![],
        };

        let (hours, notes) = if self.mode != FormMode::Start {
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
            (None, None)
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

    fn select_project_inner(&mut self) -> Option<u64> {
        let pid = self.selected_project().map(|a| a.project.id);
        self.selected_project_id = pid;
        self.tasks_loading = true;
        self.tasks.clear();
        pid
    }

    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme, tick: u64) {
        if !self.visible {
            return;
        }

        if !self.loaded {
            crate::loading::render_harv_loading(area, f, tick, "Loading projects...", theme);
            return;
        }

        let popup = if self.mode != FormMode::Start {
            crate::popup::centered_rect_fixed(area.width.saturating_sub(6).min(72), 22, area)
        } else {
            crate::popup::centered_rect_fixed(area.width.saturating_sub(6).min(60), 14, area)
        };
        f.render_widget(Clear, popup);

        let title = match self.mode {
            FormMode::Start => " Start Timer ",
            FormMode::Create => " New Entry ",
            FormMode::Edit => " Edit Entry ",
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

        if self.mode != FormMode::Start {
            let layout = Layout::vertical([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(inner);

            self.render_project_section(layout[0], inner_width, content_x, f, theme);
            self.render_task_section(layout[1], inner_width, content_x, f, theme);
            self.render_text_field(
                "Date (YYYY-MM-DD)",
                &self.date,
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
                "Hours (e.g. 1.5 or 1:30)",
                &self.hours,
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
                "Notes (optional)",
                &self.notes,
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
                " Tab: next field │ Enter: submit │ Esc: cancel ",
                Style::new().fg(theme.muted),
            );
            f.render_widget(Paragraph::new(help), layout[5]);
        } else {
            let layout = Layout::vertical([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);

            self.render_project_section(layout[0], inner_width, content_x, f, theme);
            self.render_task_section(layout[1], inner_width, content_x, f, theme);

            let help = Span::styled(
                " Tab: next field │ Enter: start timer │ Esc: cancel ",
                Style::new().fg(theme.muted),
            );
            f.render_widget(Paragraph::new(help), layout[3]);
        }
    }

    fn render_project_section(
        &mut self,
        area: Rect,
        width: u16,
        x: u16,
        f: &mut Frame,
        theme: &Theme,
    ) {
        let active = self.active == Field::ProjectList;
        let border_style = if active {
            Style::new().fg(theme.primary)
        } else {
            Style::new().fg(theme.border)
        };

        let title = if !self.project_search.is_empty() {
            format!(" Project [{}] ", self.project_search)
        } else {
            " Project ".into()
        };

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let items: Vec<Line> = self
            .filtered_assignments
            .iter()
            .map(|&i| {
                let a = &self.assignments[i];
                let client = a
                    .client
                    .as_ref()
                    .map(|c| format!("{} · ", c.name))
                    .unwrap_or_default();
                let text = format!("{} {}", a.project.name, client);
                if active {
                    Line::from(Span::styled(text, Style::new().fg(theme.fg)))
                } else {
                    Line::from(Span::styled(text, Style::new().fg(theme.muted)))
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

        f.render_stateful_widget(list, list_area, &mut self.project_list);
    }

    fn render_task_section(
        &mut self,
        area: Rect,
        width: u16,
        x: u16,
        f: &mut Frame,
        theme: &Theme,
    ) {
        let active = self.active == Field::TaskList;
        let border_style = if active {
            Style::new().fg(theme.primary)
        } else {
            Style::new().fg(theme.border)
        };

        let block = Block::new()
            .title(" Task ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let list_text: Vec<String> = if self.tasks_loading {
            vec!["Loading...".into()]
        } else if self.tasks.is_empty() && self.selected_project_id.is_some() {
            vec!["No tasks available".into()]
        } else if self.selected_project_id.is_none() {
            vec!["Select a project first".into()]
        } else {
            self.tasks
                .iter()
                .map(|t| {
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

    fn render_text_field(
        &self,
        label: &str,
        value: &str,
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
            Span::styled("(empty)", Style::new().fg(theme.muted))
        } else {
            Span::styled(value.to_string(), Style::new().fg(theme.fg))
        };

        let inner = block.inner(area);
        f.render_widget(&block, area);
        f.render_widget(Paragraph::new(display), inner);
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
                if let Some(pid) = self.select_project_inner() {
                    return vec![Action::FormSelectProject(pid)];
                }
                vec![]
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self
                    .project_list
                    .selected()
                    .map_or(0, |i| i.saturating_sub(1));
                self.project_list.select(Some(i));
                if let Some(pid) = self.select_project_inner() {
                    return vec![Action::FormSelectProject(pid)];
                }
                vec![]
            }
            KeyCode::Enter => {
                if let Some(pid) = self.select_project_inner() {
                    self.active = Field::TaskList;
                    return vec![Action::FormSelectProject(pid)];
                }
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
                self.active = if self.mode != FormMode::Start {
                    Field::Date
                } else {
                    Field::ProjectList
                };
                vec![]
            }
            KeyCode::BackTab => {
                self.active = Field::ProjectList;
                vec![]
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let max = self.tasks.len().saturating_sub(1);
                let i = self.task_list.selected().map_or(0, |i| (i + 1).min(max));
                self.task_list.select(Some(i));
                vec![]
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self.task_list.selected().map_or(0, |i| i.saturating_sub(1));
                self.task_list.select(Some(i));
                vec![]
            }
            KeyCode::Enter => self.submit_entry(),
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
                self.active = if self.mode != FormMode::Start {
                    match self.active {
                        Field::Date => Field::Hours,
                        Field::Hours => Field::Notes,
                        Field::Notes => Field::ProjectList,
                        _ => Field::ProjectList,
                    }
                } else {
                    Field::ProjectList
                };
                vec![]
            }
            KeyCode::BackTab => {
                self.active = if self.mode != FormMode::Start {
                    match self.active {
                        Field::Date => Field::TaskList,
                        Field::Hours => Field::Date,
                        Field::Notes => Field::Hours,
                        _ => Field::ProjectList,
                    }
                } else {
                    Field::TaskList
                };
                vec![]
            }
            KeyCode::Enter => self.submit_entry(),
            KeyCode::Backspace => {
                self.active_text_field_mut().pop();
                vec![]
            }
            KeyCode::Char(c) => {
                self.active_text_field_mut().push(c);
                vec![]
            }
            _ => vec![],
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
        let f = TimeEntryForm::new(None, None, None, FormMode::Start, None, None, None, None);
        assert!(matches!(f.mode, FormMode::Start));
        assert!(!f.loaded);
        assert!(f.selected_project_id.is_none());
        assert!(f.hours.is_empty());
        assert!(f.notes.is_empty());
    }

    #[test]
    fn test_new_create_form() {
        let f = TimeEntryForm::new(None, None, None, FormMode::Create, None, None, None, None);
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
        );
        let assignments = vec![
            project_assignment(5, "Alpha"),
            project_assignment(10, "Beta"),
        ];
        let pid = f.update_assignments(assignments);
        assert_eq!(pid, Some(10));
        assert!(f.loaded);
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
        );
        let assignments = vec![project_assignment(5, "Alpha")];
        let pid = f.update_assignments(assignments);
        assert_eq!(pid, None);
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
        let mut f = TimeEntryForm::new(None, None, None, FormMode::Start, None, None, None, None);
        f.selected_project_id = Some(10);
        let tasks = vec![task_assignment(20, "Design")];
        f.update_tasks(tasks);
        assert_eq!(f.task_list.selected(), Some(0));
    }

    #[test]
    fn test_project_search_filters() {
        let mut f = TimeEntryForm::new(None, None, None, FormMode::Start, None, None, None, None);
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
        let mut f = TimeEntryForm::new(None, None, None, FormMode::Start, None, None, None, None);
        assert_eq!(f.active, Field::ProjectList);

        // ProjectList Tab -> TaskList
        let _ = f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::TaskList);

        // TaskList Tab -> ProjectList (start mode, no Date field)
        let _ = f.handle_key(&key_press(KeyCode::Tab));
        assert_eq!(f.active, Field::ProjectList);
    }

    #[test]
    fn test_tab_cycles_fields_in_create_mode() {
        let mut f = TimeEntryForm::new(None, None, None, FormMode::Create, None, None, None, None);
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
        let mut f = TimeEntryForm::new(None, None, None, FormMode::Start, None, None, None, None);
        assert!(f.visible);
        let actions = f.handle_key(&key_press(KeyCode::Esc));
        assert!(!f.visible);
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_submit_requires_project_and_task() {
        let mut f = TimeEntryForm::new(None, None, None, FormMode::Start, None, None, None, None);
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
        );
        f.assignments = vec![project_assignment(10, "Beta")];
        f.filtered_assignments = vec![0];
        f.project_list.select(Some(0));
        f.selected_project_id = Some(10);

        f.tasks = vec![task_assignment(20, "Development")];
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
        );
        f.assignments = vec![project_assignment(10, "Beta")];
        f.filtered_assignments = vec![0];
        f.project_list.select(Some(0));
        f.selected_project_id = Some(10);
        f.tasks = vec![task_assignment(20, "Development")];
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
            None,
        );
        f.assignments = vec![project_assignment(10, "Beta")];
        f.filtered_assignments = vec![0];
        f.project_list.select(Some(0));
        f.selected_project_id = Some(10);
        f.tasks = vec![task_assignment(20, "Development")];
        f.task_list.select(Some(0));

        let actions = f.submit_entry();
        assert!(matches!(
            actions[0],
            Action::CreateEntry {
                hours: None,
                notes: None,
                ..
            }
        ));
    }
}
