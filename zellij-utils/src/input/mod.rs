//! The way terminal input is handled.

pub mod actions;
pub mod command;
pub mod config;
pub mod keybinds;
pub mod layout;
pub mod mouse;
pub mod options;
pub mod theme;

use zellij_tile::data::{InputMode, Key, ModeInfo, Palette, PluginCapabilities};

/// Creates a [`ModeInfo`] struct indicating the current [`InputMode`] and its keybinds
/// (as pairs of [`String`]s).
pub fn get_mode_info(
    mode: InputMode,
    palette: Palette,
    capabilities: PluginCapabilities,
) -> ModeInfo {
    let keybinds = match mode {
        InputMode::Normal | InputMode::Locked => Vec::new(),
        InputMode::Resize => vec![("←↓↑→".to_string(), "Resize".to_string())],
        InputMode::Pane => vec![
            ("←↓↑→".to_string(), "Move focus".to_string()),
            ("p".to_string(), "Next".to_string()),
            ("n".to_string(), "New".to_string()),
            ("d".to_string(), "Down split".to_string()),
            ("r".to_string(), "Right split".to_string()),
            ("x".to_string(), "Close".to_string()),
            ("f".to_string(), "Fullscreen".to_string()),
            ("z".to_string(), "Frames".to_string()),
        ],
        InputMode::Tab => vec![
            ("←↓↑→".to_string(), "Move focus".to_string()),
            ("n".to_string(), "New".to_string()),
            ("x".to_string(), "Close".to_string()),
            ("r".to_string(), "Rename".to_string()),
            ("s".to_string(), "Sync".to_string()),
            ("Tab".to_string(), "Toggle".to_string()),
        ],
        InputMode::Scroll => vec![
            ("↓↑".to_string(), "Scroll".to_string()),
            ("PgUp/PgDn".to_string(), "Scroll Page".to_string()),
        ],
        InputMode::RenameTab => vec![("Enter".to_string(), "when done".to_string())],
        InputMode::Session => vec![("d".to_string(), "Detach".to_string())],
    };

    let session_name = std::env::var("ZELLIJ_SESSION_NAME").ok();

    ModeInfo {
        mode,
        keybinds,
        palette,
        capabilities,
        session_name,
    }
}

pub fn parse_keys(input_bytes: &[u8]) -> Vec<Key> {
    let keys = Vec::new();
    loop {
        let event: crossterm::Result<crossterm::event::Event> =
            todo!("crossterm won't let us parse stuff directly from a byte slice");
        match event {
            Ok(event) => keys.push(cast_crossterm_event(event)),
            Err(_) => break, // Assume this is end of stream
        }
    }
    keys
}

// FIXME: This is an absolutely cursed function that should be destroyed as soon
// as an alternative that doesn't touch zellij-tile can be developed...
pub fn cast_crossterm_event(event: crossterm::event::Event) -> Key {
    use crossterm::event::Event;
    match event {
        Event::Key(key) => cast_crossterm_key(key),
        _ => {
            unimplemented!("Encountered an unknown event type!")
        }
    }
}

pub fn cast_crossterm_key(event: crossterm::event::KeyEvent) -> Key {
    use crossterm::event::KeyModifiers;
    let key = cast_crossterm_key_code(event.code);
    // TODO: special handling for shift? (At least mask it out so it doesn't put us into the unimplemented arms?)
    match event.modifiers {
        KeyModifiers::NONE => key,
        KeyModifiers::SHIFT => key,
        KeyModifiers::CONTROL => match key {
            Key::Char(c) => Key::Ctrl(c),
            _ => unimplemented!("Unexpected modified event"),
        },
        KeyModifiers::ALT => match key {
            Key::Char(c) => Key::Alt(c),
            _ => unimplemented!("Unexpected modified event"),
        },
        _ => unimplemented!("Unhandled modifier combination"),
    }
}

fn cast_crossterm_key_code(code: crossterm::event::KeyCode) -> Key {
    use crossterm::event::KeyCode;
    match code {
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Char('\n'), // TODO: is this correct?
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Tab => Key::Char('\t'), // TODO: is this correct?
        KeyCode::BackTab => Key::BackTab,
        KeyCode::Delete => Key::Delete,
        KeyCode::Insert => Key::Insert,
        KeyCode::F(n) => Key::F(n),
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Null => Key::Null,
        KeyCode::Esc => Key::Esc,
    }
}

// TODO: make a trait impl out of this?
pub fn cast_key_to_crossterm(event: Key) -> crossterm::event::KeyEvent {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let plain = |code| KeyEvent::new(code, KeyModifiers::NONE);
    // TODO: special handling for shift? (At least mask it out so it doesn't put us into the unimplemented arms?)
    match event {
        Key::Backspace => plain(KeyCode::Backspace),
        Key::Left => plain(KeyCode::Left),
        Key::Right => plain(KeyCode::Right),
        Key::Up => plain(KeyCode::Up),
        Key::Down => plain(KeyCode::Down),
        Key::Home => plain(KeyCode::Home),
        Key::End => plain(KeyCode::End),
        Key::PageUp => plain(KeyCode::PageUp),
        Key::PageDown => plain(KeyCode::PageDown),
        Key::BackTab => plain(KeyCode::BackTab),
        Key::Delete => plain(KeyCode::Delete),
        Key::Insert => plain(KeyCode::Insert),
        Key::F(n) => plain(KeyCode::F(n)),
        Key::Char('\n') => plain(KeyCode::Enter),
        Key::Char('\t') => plain(KeyCode::Tab),
        Key::Char(c) => plain(KeyCode::Char(c)),
        Key::Null => plain(KeyCode::Null),
        Key::Esc => plain(KeyCode::Esc),
    }
}
