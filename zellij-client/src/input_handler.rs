//! Main input logic.

use zellij_utils::{
    input::{
        mouse::{MouseButton, MouseEvent},
        options::Options,
    },
    zellij_tile,
};

use crate::{os_input_output::ClientOsApi, ClientInstruction, CommandIsExecuting};
use zellij_utils::{
    channels::{SenderWithContext, OPENCALLS},
    crossterm,
    errors::ContextType,
    input::{actions::Action, cast_crossterm_key, config::Config, keybinds::Keybinds},
    ipc::{ClientToServerMsg, ExitReason},
};

use zellij_tile::data::{InputMode, Key};

/// Handles the dispatching of [`Action`]s according to the current
/// [`InputMode`], and keep tracks of the current [`InputMode`].
struct InputHandler {
    /// The current input mode
    mode: InputMode,
    os_input: Box<dyn ClientOsApi>,
    config: Config,
    options: Options,
    command_is_executing: CommandIsExecuting,
    send_client_instructions: SenderWithContext<ClientInstruction>,
    should_exit: bool,
    pasting: bool,
}

impl InputHandler {
    /// Returns a new [`InputHandler`] with the attributes specified as arguments.
    fn new(
        os_input: Box<dyn ClientOsApi>,
        command_is_executing: CommandIsExecuting,
        config: Config,
        options: Options,
        send_client_instructions: SenderWithContext<ClientInstruction>,
        mode: InputMode,
    ) -> Self {
        InputHandler {
            mode,
            os_input,
            config,
            options,
            command_is_executing,
            send_client_instructions,
            should_exit: false,
            pasting: false,
        }
    }

    /// Main input event loop. Interprets the terminal [`Event`](crossterm::event::Event)s
    /// as [`Action`]s according to the current [`InputMode`], and dispatches those actions.
    fn handle_input(&mut self) {
        use crossterm::event::Event;
        let mut err_ctx = OPENCALLS.with(|ctx| *ctx.borrow());
        err_ctx.add_call(ContextType::StdinHandler);
        // TODO: still using this and the pasting flag?
        let bracketed_paste_start = vec![27, 91, 50, 48, 48, 126]; // \u{1b}[200~
        let bracketed_paste_end = vec![27, 91, 50, 48, 49, 126]; // \u{1b}[201

        if !self.options.disable_mouse_mode {
            // TODO: needs work
            self.os_input.enable_mouse();
        }
        loop {
            if self.should_exit {
                break;
            }
            match crossterm::event::read() {
                Ok(event) => match event {
                    Event::Key(key) => {
                        let key = cast_crossterm_key(key);
                        self.handle_key(&key);
                    }
                    Event::Mouse(me) => {
                        let mouse_event = zellij_utils::input::mouse::MouseEvent::from(me);
                        self.handle_mouse_event(&mouse_event);
                    }
                    Event::Resize(_cols, _rows) => todo!(),
                },
                Err(err) => panic!("Encountered read error: {:?}", err),
            }
        }
    }
    fn handle_key(&mut self, key: &Key) {
        let keybinds = &self.config.keybinds;
        if self.pasting {
            // we're inside a paste block, if we're in a mode that allows sending text to the
            // terminal, send all text directly without interpreting it
            // otherwise, just discard the input
            if self.mode == InputMode::Normal || self.mode == InputMode::Locked {
                let action = Action::Write(todo!());
                self.dispatch_action(action);
            }
        } else {
            for action in Keybinds::key_to_actions(key, &self.mode, keybinds) {
                let should_exit = self.dispatch_action(action);
                if should_exit {
                    self.should_exit = true;
                }
            }
        }
    }
    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent) {
        match *mouse_event {
            MouseEvent::Press(button, point) => match button {
                MouseButton::WheelUp => {
                    self.dispatch_action(Action::ScrollUpAt(point));
                }
                MouseButton::WheelDown => {
                    self.dispatch_action(Action::ScrollDownAt(point));
                }
                MouseButton::Left => {
                    self.dispatch_action(Action::LeftClick(point));
                }
                _ => {}
            },
            MouseEvent::Release(point) => {
                self.dispatch_action(Action::MouseRelease(point));
            }
            MouseEvent::Hold(point) => {
                self.dispatch_action(Action::MouseHold(point));
                self.os_input
                    .start_action_repeater(Action::MouseHold(point));
            }
        }
    }

    /// Dispatches an [`Action`].
    ///
    /// This function's body dictates what each [`Action`] actually does when
    /// dispatched.
    ///
    /// # Return value
    /// Currently, this function returns a boolean that indicates whether
    /// [`Self::handle_input()`] should break after this action is dispatched.
    /// This is a temporary measure that is only necessary due to the way that the
    /// framework works, and shouldn't be necessary anymore once the test framework
    /// is revised. See [issue#183](https://github.com/zellij-org/zellij/issues/183).
    fn dispatch_action(&mut self, action: Action) -> bool {
        let mut should_break = false;

        match action {
            Action::Quit | Action::Detach => {
                self.os_input
                    .send_to_server(ClientToServerMsg::Action(action));
                self.exit();
                should_break = true;
            }
            Action::SwitchToMode(mode) => {
                self.mode = mode;
                self.os_input
                    .send_to_server(ClientToServerMsg::Action(action));
            }
            Action::CloseFocus
            | Action::NewPane(_)
            | Action::NewTab(_)
            | Action::GoToNextTab
            | Action::GoToPreviousTab
            | Action::CloseTab
            | Action::GoToTab(_)
            | Action::ToggleTab
            | Action::MoveFocusOrTab(_) => {
                self.command_is_executing.blocking_input_thread();
                self.os_input
                    .send_to_server(ClientToServerMsg::Action(action));
                self.command_is_executing
                    .wait_until_input_thread_is_unblocked();
            }
            _ => self
                .os_input
                .send_to_server(ClientToServerMsg::Action(action)),
        }

        should_break
    }

    /// Routine to be called when the input handler exits (at the moment this is the
    /// same as quitting Zellij).
    fn exit(&mut self) {
        self.send_client_instructions
            .send(ClientInstruction::Exit(ExitReason::Normal))
            .unwrap();
    }
}

/// Entry point to the module. Instantiates an [`InputHandler`] and starts
/// its [`InputHandler::handle_input()`] loop.
pub(crate) fn input_loop(
    os_input: Box<dyn ClientOsApi>,
    config: Config,
    options: Options,
    command_is_executing: CommandIsExecuting,
    send_client_instructions: SenderWithContext<ClientInstruction>,
    default_mode: InputMode,
) {
    let _handler = InputHandler::new(
        os_input,
        command_is_executing,
        config,
        options,
        send_client_instructions,
        default_mode,
    )
    .handle_input();
}

#[cfg(test)]
#[path = "./unit/input_handler_tests.rs"]
mod grid_tests;
