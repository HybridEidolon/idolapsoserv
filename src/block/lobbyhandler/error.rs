//! Error struct/enum for lobbies.

use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LobbyError {
    /// The operation failed because the lobby is full.
    IsFull,
    /// The player specified is already in this lobby.
    AlreadyInLobby,
    /// The player specified is not in this lobby.
    NotInLobby,
    /// An IO error occurred.
    Io
}

impl From<LobbyError> for io::Error {
    fn from(val: LobbyError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, format!("{:?}", val))
    }
}
