
use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, prelude::*};
use std::env;
use std::fs;
use std::io::{self};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

mod db_helper;
use db_helper::db_helper as db_mgr;
mod run_py_struct;
use run_py_struct::run_py_struct::RunPy;

const DB_PATH: &str = "./data/db.json";

// Define a mutable global string using `lazy_static` and `Mutex`
lazy_static! {
    static ref DATABASE_ADDR: Mutex<String> = Mutex::new(String::from(DB_PATH));
    static ref PYTHON_BIND: Mutex<String> = Mutex::new(String::from("python3"));
    static ref ACTION_MSG: Mutex<String> = Mutex::new(String::from("Nothing being pressed yet..."));
    static ref LOG_MSG: Mutex<String> = Mutex::new(String::from("Initial log."));
    static ref READY_TO_INIT_DB: Mutex<bool> = Mutex::new(false);
}
fn set_db_addr(new_value: &str) {
    // Lock the mutex to mutate the global string
    let mut global = DATABASE_ADDR.lock().unwrap();
    *global = new_value.to_string(); // Update the global string
}
fn set_py_bind(new_value: &str) {
    // Lock the mutex to mutate the global string
    let mut global = PYTHON_BIND.lock().unwrap();
    *global = new_value.to_string(); // Update the global string
}
fn set_action_msg(new_value: &str) {
    // Lock the mutex to mutate the global string
    let mut global = ACTION_MSG.lock().unwrap();
    *global = new_value.to_string(); // Update the global string
}

fn set_log_msg(new_value: &str) {
    // Lock the mutex to mutate the global string
    let mut global = LOG_MSG.lock().unwrap();
    *global = new_value.to_string(); // Update the global string
}

fn set_init_db_status(new_value: bool) {
    // Lock the mutex to mutate the global string
    let mut global = READY_TO_INIT_DB.lock().unwrap();
    *global = new_value; // Update the global string
}


// Handling I/O Errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuItem {
    Home,
    RunPy,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::RunPy => 1,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read .env
    if let Ok(env_vars) = db_mgr::read_env_file(".env") {
        if let Some(value) = env_vars.get("DATABASE_ADDR") {
            println!("Value for DATABASE_ADDR: {}", value);
            set_db_addr(value.as_str());
        } else {
            println!("DATABASE_ADDR default to {}", DATABASE_ADDR.lock().unwrap().clone());
        }
        if let Some(value) = env_vars.get("PYTHON_BIND") {
            println!("Value for PYTHON_BIND: {}", value);
            set_py_bind(value.as_str());
        } else {
            println!("PYTHON_BIND default to {}", PYTHON_BIND.lock().unwrap().clone());
        }
    } else {
        eprintln!("Failed to read .env file");
    }

    enable_raw_mode().expect("can run in raw mode");

    // Initialize json database
    db_mgr::seed_database(DATABASE_ADDR.lock().unwrap().clone());

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "RunPy", "Add", "Delete", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut py_list_state = ListState::default();
    py_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        // Cut Sections
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                        Constraint::Length(12),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let copyright =
                Paragraph::new("Fork from pet-CLI 2020")
                    .style(Style::default().fg(Color::LightCyan))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::White))
                            .title("Copyright")
                            .border_type(BorderType::Plain),
                    );
            let action_msg = Paragraph::new(ACTION_MSG.lock().unwrap().clone())
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Your last input action")
                        .border_type(BorderType::Plain),
                );
            let log_msg = Paragraph::new(LOG_MSG.lock().unwrap().clone())
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Your last log")
                        .border_type(BorderType::Plain),
                );
            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::RunPy => {
                    let run_py_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_scripts(&py_list_state);
                    rect.render_stateful_widget(left, run_py_chunks[0], &mut py_list_state);
                    rect.render_widget(right, run_py_chunks[1]);
                    rect.render_widget(action_msg.clone(), chunks[2]);
                }
            }
            rect.render_widget(action_msg, chunks[2]);
            rect.render_widget(log_msg, chunks[3]);
            rect.render_widget(copyright, chunks[4]);
        })?;

        match rx.recv()? {
            Event::Input(event) => {
                // show pressed key
                let format_string = format!("pressed --- {:?}", event.code);
                let pressed_key: &str = format_string.as_str();
                set_action_msg(pressed_key);

                match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                    KeyCode::Char('r') => active_menu_item = MenuItem::RunPy,
                    KeyCode::Char('a') => {
                        add_random_script_to_db().expect("can add new random script");
                    }
                    KeyCode::Char('d') => {
                        remove_script_at_index(&mut py_list_state).expect("can remove script");
                    }
                    KeyCode::Down => {
                        if let Some(selected) = py_list_state.selected() {
                            let amount_scripts = read_db().expect("can fetch script list").len();
                            if selected >= amount_scripts - 1 {
                                py_list_state.select(Some(0));  // move to first
                            } else {
                                py_list_state.select(Some(selected + 1));
                            }
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = py_list_state.selected() {
                            let amount_scripts = read_db().expect("can fetch script list").len();
                            if selected > 0 {
                                py_list_state.select(Some(selected - 1));
                            } else {
                                py_list_state.select(Some(amount_scripts - 1));  // reach the top, move back to last one
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if active_menu_item == MenuItem::RunPy {
                            if let Some(selected) = py_list_state.selected() {
                                let content_cli = read_db().expect("can fetch script list");
                                let mut entered_script: String = content_cli.get(selected).unwrap().py_script.clone();
                                if is_py_in_current_folder(&entered_script.as_str()){
                                    set_action_msg(format!("Executed --- {:?}", &entered_script).as_str());
                                } else{
                                    entered_script = String::from("default_script.py");
                                    set_action_msg(format!("Executed --- default_script.py").as_str());
                                }
                                // need to solve async
                                let run_py = Command::new(PYTHON_BIND.lock().unwrap().clone())
                                    .arg(entered_script) // Assuming you have a variable named entered_script
                                    .stdout(Stdio::piped()) // Capture stdout
                                    .stderr(Stdio::piped()) // Capture stderr
                                    .output()
                                    .expect("Failed to execute py");

                                // if command was successful, print stdout, otherwise print stderr
                                let py_output = if run_py.status.success() {
                                    String::from_utf8_lossy(&run_py.stdout).to_string()
                                } else {
                                    String::from_utf8_lossy(&run_py.stderr).to_string()
                                };
                                set_log_msg(format!("{}", py_output).as_str());
                            }
                        }
                    }
                    // a flip-flop for init flag
                    KeyCode::Char('i') => {
                        if !*READY_TO_INIT_DB.lock().expect("false") {
                            set_log_msg("InitDB Flag is ON. Do you want to initialize DB?\nPress y at anytime to proceed.\nPress i again to remove InitDB Flag.");
                            set_init_db_status(true);
                        } else{
                            set_log_msg("InitDB Flag is OFF");
                            set_init_db_status(false);
                        }
                    }
                    KeyCode::Char('y') => {
                        if *READY_TO_INIT_DB.lock().expect("false") {
                            py_list_state.select(Some(0));  // remove all will cause selected not match
                            set_log_msg("You Initialized json DB");
                            let _init_status = db_mgr::overwrite_json( DATABASE_ADDR.lock().unwrap().clone());
                            set_init_db_status(false);
                        }
                    }
                    _ => {}
                }
            }
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Run-Py-CLI",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Place your routine py files in root folder.")]),
        Spans::from(vec![Span::raw("Press 'r' to visit RunPy page.")]),
        Spans::from(vec![Span::raw("Press 'Enter' to run selected script.")]),
        Spans::from(vec![Span::raw("Press 'd' to delete selected script")]),
        Spans::from(vec![Span::raw("Press 'i' to initialize the json database.")]),
        ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn render_scripts<'a>(py_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let py_scripts = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("PyScripts")
        .border_type(BorderType::Plain);

    let script_list = read_db().unwrap_or_else(|_err| {
        panic!("Can't fetch py list. Please proceeed with init <i>.")
    });
    let items: Vec<_> = script_list
        .iter()
        .map(|py_script| {
            ListItem::new(Spans::from(vec![Span::styled(
                py_script.py_script.clone(),  // show in Letf Screen
                Style::default(),
            )]))
        })
        .collect();

    let selected_script = script_list
        .get(
            py_list_state
                .selected()
                .expect("there is always a selected script"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(py_scripts).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let run_py_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_script.id.to_string())),
        Cell::from(Span::raw(selected_script.py_script)),
        Cell::from(Span::raw(selected_script.description)),
        Cell::from(Span::raw(selected_script.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "PyScripts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Description",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ]);

    (list, run_py_detail)
}

fn read_db() -> Result<Vec<RunPy>, Error> {
    let db_content = fs::read_to_string(DATABASE_ADDR.lock().unwrap().clone())?;
    let parsed: Vec<RunPy> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn add_random_script_to_db() -> Result<Vec<RunPy>, Error> {
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(DATABASE_ADDR.lock().unwrap().clone())?;
    let mut parsed: Vec<RunPy> = serde_json::from_str(&db_content)?;
    let rand_description: String = rng
        .sample_iter(Alphanumeric)
        .take(5) // Take n characters
        .map(char::from) // Convert to `char`
        .collect(); // Collect into a `String`
    let rand_py: String= format!("Rand_{}.py", &rand_description);
    let catsdogs = match rng.gen_range(0, 2) {
            0 => "script.py",
            _ => &rand_py,
        };
    let random_script = RunPy {
        id: rng.gen_range(0, 99),
        py_script: catsdogs.to_owned(),
        description: format!("mock$ {}", rand_description),
        created_at: Utc::now(),
    };

    parsed.push(random_script);
    fs::write(DATABASE_ADDR.lock().unwrap().clone().as_str(), &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

fn remove_script_at_index(py_list_state: &mut ListState) -> Result<(), Error> {
    if let Some(selected) = py_list_state.selected() {
        let db_content = fs::read_to_string(DATABASE_ADDR.lock().unwrap().clone())?;
        let mut parsed: Vec<RunPy> = serde_json::from_str(&db_content)?;

        if parsed.len() == 1 {
            set_action_msg("DB has only one element left. Operation skipped.");
            return Ok(());
        }
        parsed.remove(selected);
        fs::write(DATABASE_ADDR.lock().unwrap().clone(), &serde_json::to_vec(&parsed)?)?;
        if selected > 0 {
            py_list_state.select(Some(selected - 1));
        } else {
            py_list_state.select(Some(0));
        }
    }
    Ok(())
}

fn is_py_in_current_folder(name: &str) -> bool {
    let current_dir = env::current_dir().expect("Failed to get current dir");
    let py_path = current_dir.join(name);
    if Path::new(&py_path).exists(){
        return true
    }
    false
}

