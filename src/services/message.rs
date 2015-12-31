//! Wrappers for the various message namespaces in `psomsg`

use psomsg::bb::Message as BbMsg;
use psomsg::patch::Message as PatchMsg;

/// A wrapper around all the possible message types.
#[derive(Clone)]
pub enum NetMsg {
    Bb(BbMsg),
    Patch(PatchMsg)
}

impl Into<NetMsg> for PatchMsg {
    fn into(self) -> NetMsg {
        NetMsg::Patch(self)
    }
}

impl Into<NetMsg> for BbMsg {
    fn into(self) -> NetMsg {
        NetMsg::Bb(self)
    }
}
