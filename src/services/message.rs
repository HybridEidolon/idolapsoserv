//! Wrappers for the various message namespaces in `psomsg`

use psomsg::bb::Message as BbMsg;
use psomsg::patch::Message as PatchMsg;

/// A wrapper around all the possible message types.
#[derive(Clone)]
pub enum NetMsg {
    Bb(BbMsg),
    Patch(PatchMsg)
}

impl From<BbMsg> for NetMsg {
    fn from(val: BbMsg) -> NetMsg {
        NetMsg::Bb(val)
    }
}

impl From<PatchMsg> for NetMsg {
    fn from(val: PatchMsg) -> NetMsg {
        NetMsg::Patch(val)
    }
}
