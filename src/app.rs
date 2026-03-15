use crate::server::Server;
use std::path::PathBuf;

/// Which screen the app is on
#[derive(Debug)]
pub enum Screen {
    Dashboard,
    AddForm,
    EditForm(usize),
    ConfirmDelete(usize),
    SshSession(crate::ssh::SshSession),
}

/// Which field is focused in the form
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormField {
    Alias,
    Host,
    Username,
    Port,
    Tags,
}

impl FormField {
    pub fn next(self) -> Self {
        match self {
            Self::Alias => Self::Host,
            Self::Host => Self::Username,
            Self::Username => Self::Port,
            Self::Port => Self::Tags,
            Self::Tags => Self::Alias,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Alias => Self::Tags,
            Self::Host => Self::Alias,
            Self::Username => Self::Host,
            Self::Port => Self::Username,
            Self::Tags => Self::Port,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Alias => "Alias",
            Self::Host => "Host / IP",
            Self::Username => "Username",
            Self::Port => "Port",
            Self::Tags => "Tags / Notes",
        }
    }
}

pub const FORM_FIELDS: [FormField; 5] = [
    FormField::Alias,
    FormField::Host,
    FormField::Username,
    FormField::Port,
    FormField::Tags,
];

/// Holds the text in each form field
#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub alias: String,
    pub host: String,
    pub username: String,
    pub port: String,
    pub tags: String,
    pub focused: FormField,
}

impl Default for FormField {
    fn default() -> Self {
        Self::Alias
    }
}

impl FormState {
    pub fn new_empty() -> Self {
        Self {
            port: "22".to_string(),
            ..Default::default()
        }
    }

    pub fn from_server(s: &Server) -> Self {
        Self {
            alias: s.alias.clone(),
            host: s.host.clone(),
            username: s.username.clone(),
            port: s.port.to_string(),
            tags: s.tags.clone(),
            focused: FormField::Alias,
        }
    }

    pub fn get_field(&self, f: FormField) -> &str {
        match f {
            FormField::Alias => &self.alias,
            FormField::Host => &self.host,
            FormField::Username => &self.username,
            FormField::Port => &self.port,
            FormField::Tags => &self.tags,
        }
    }

    pub fn get_field_mut(&mut self, f: FormField) -> &mut String {
        match f {
            FormField::Alias => &mut self.alias,
            FormField::Host => &mut self.host,
            FormField::Username => &mut self.username,
            FormField::Port => &mut self.port,
            FormField::Tags => &mut self.tags,
        }
    }

    pub fn to_server(&self) -> Option<Server> {
        let alias = self.alias.trim().to_string();
        let host = self.host.trim().to_string();
        if alias.is_empty() || host.is_empty() {
            return None;
        }
        let port: u16 = self.port.trim().parse().unwrap_or(22);
        Some(Server {
            alias,
            host,
            username: self.username.trim().to_string(),
            port,
            tags: self.tags.trim().to_string(),
        })
    }
}

/// Main application state
pub struct App {
    pub servers: Vec<Server>,
    pub selected: usize,
    pub screen: Screen,
    pub form: FormState,
    pub config_path: PathBuf,
    pub should_quit: bool,
    pub status_msg: String,
    pub should_ssh: Option<usize>,
}

impl App {
    pub fn new(config_path: PathBuf) -> Self {
        let servers = crate::server::load_servers(&config_path);
        Self {
            servers,
            selected: 0,
            screen: Screen::Dashboard,
            form: FormState::new_empty(),
            config_path,
            should_quit: false,
            status_msg: String::new(),
            should_ssh: None,
        }
    }

    pub fn save(&mut self) {
        if let Err(e) = crate::server::save_servers(&self.config_path, &self.servers) {
            self.status_msg = format!("Save error: {}", e);
        } else {
            self.status_msg = "Saved.".to_string();
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.servers.is_empty() {
            self.selected = (self.selected + 1) % self.servers.len();
        }
    }

    pub fn move_selection_up(&mut self) {
        if !self.servers.is_empty() {
            self.selected = if self.selected == 0 {
                self.servers.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn start_add(&mut self) {
        self.form = FormState::new_empty();
        self.screen = Screen::AddForm;
    }

    pub fn start_edit(&mut self) {
        if let Some(s) = self.servers.get(self.selected) {
            self.form = FormState::from_server(s);
            self.screen = Screen::EditForm(self.selected);
        }
    }

    pub fn confirm_delete(&mut self) {
        if !self.servers.is_empty() {
            self.screen = Screen::ConfirmDelete(self.selected);
        }
    }

    pub fn do_delete(&mut self, idx: usize) {
        if idx < self.servers.len() {
            let alias = self.servers[idx].alias.clone();
            self.servers.remove(idx);
            self.save();
            if self.selected >= self.servers.len() && self.selected > 0 {
                self.selected -= 1;
            }
            self.status_msg = format!("Deleted '{}'", alias);
        }
        self.screen = Screen::Dashboard;
    }

    pub fn submit_form(&mut self) {
        if let Some(server) = self.form.to_server() {
            match &self.screen {
                Screen::AddForm => {
                    self.status_msg = format!("Added '{}'", server.alias);
                    self.servers.push(server);
                    self.save();
                }
                Screen::EditForm(idx) => {
                    let idx = *idx;
                    if idx < self.servers.len() {
                        self.status_msg = format!("Updated '{}'", server.alias);
                        self.servers[idx] = server;
                        self.save();
                    }
                }
                _ => {}
            }
            self.screen = Screen::Dashboard;
        } else {
            self.status_msg = "Alias and Host are required.".to_string();
        }
    }

    pub fn cancel_form(&mut self) {
        self.screen = Screen::Dashboard;
        self.status_msg.clear();
    }

    pub fn initiate_ssh(&mut self) {
        if !self.servers.is_empty() {
            self.should_ssh = Some(self.selected);
        }
    }
}
