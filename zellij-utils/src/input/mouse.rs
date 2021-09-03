use serde::{Deserialize, Serialize};

use crate::position::Position;

/// A mouse related event
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum MouseEvent {
    /// A mouse button was pressed.
    ///
    /// The coordinates are zero-based.
    Press(MouseButton, Position),
    /// A mouse button was released.
    ///
    /// The coordinates are zero-based.
    Release(Position),
    /// A mouse button is held over the given coordinates.
    ///
    /// The coordinates are zero-based.
    Hold(Position),
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(event: crossterm::event::MouseEvent) -> Self {
        use crossterm::event::MouseEventKind;
        // TODO: still need subtractions or not?
        let (x, y) = (event.column, event.row);
        match event.kind {
            MouseEventKind::Down(button) => Self::Press(
                MouseButton::from(button),
                Position::new((y.saturating_sub(1)) as i32, x.saturating_sub(1)),
            ),
            MouseEventKind::Up(_button) => Self::Release(Position::new(
                (y.saturating_sub(1)) as i32,
                x.saturating_sub(1),
            )),
            MouseEventKind::Drag(_button) => Self::Hold(Position::new(
                (y.saturating_sub(1)) as i32,
                x.saturating_sub(1),
            )),
            MouseEventKind::Moved => todo!(),
            MouseEventKind::ScrollDown => Self::Press(
                MouseButton::WheelDown,
                Position::new((y.saturating_sub(1)) as i32, x.saturating_sub(1)),
            ),
            MouseEventKind::ScrollUp => Self::Press(
                MouseButton::WheelUp,
                Position::new((y.saturating_sub(1)) as i32, x.saturating_sub(1)),
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum MouseButton {
    /// The left mouse button.
    Left,
    /// The right mouse button.
    Right,
    /// The middle mouse button.
    Middle,
    /// Mouse wheel is going up.
    ///
    /// This event is typically only used with Mouse::Press.
    WheelUp,
    /// Mouse wheel is going down.
    ///
    /// This event is typically only used with Mouse::Press.
    WheelDown,
}

impl From<crossterm::event::MouseButton> for MouseButton {
    fn from(button: crossterm::event::MouseButton) -> Self {
        use crossterm::event::MouseButton as CButton;
        match button {
            CButton::Left => Self::Left,
            CButton::Right => Self::Right,
            CButton::Middle => Self::Middle,
        }
    }
}
