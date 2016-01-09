//! Callback manager for the shipgate client.

use super::SgSender;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use ::shipgate::msg::Message;

/// Ship gate callback manager.
pub struct SgCbMgr<H> {
    sender: SgSender,
    callbacks: Rc<RefCell<HashMap<u32, (usize, Box<FnMut(H, Message)>)>>>
}

impl<H> From<SgSender> for SgCbMgr<H> {
    fn from(val: SgSender) -> SgCbMgr<H> {
        SgCbMgr {
            sender: val,
            callbacks: Default::default()
        }
    }
}

impl<H> SgCbMgr<H> {
    pub fn request<M: Into<Message>, CB>(&mut self, cid: usize, msg: M, cb: CB) -> Result<(), String>
    where CB: FnMut(H, Message) + 'static {
        match self.sender.send(msg.into())
            .map_err(|e| format!("{}", e))
        {
            Ok(req) => {
                self.callbacks.borrow_mut().insert(req, (cid, Box::new(cb)));
                debug!("ShipGate request sent with ID {}", req);
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    pub fn send<M: Into<Message>>(&mut self, msg: M) -> Result<(), String> {
        self.sender.send_forget(msg.into())
    }

    /// Get the callback for the request given
    pub fn cb_for_req(&mut self, req: u32) -> Option<(usize, Box<FnMut(H, Message)>)> {
        self.callbacks.borrow_mut().remove(&req)
    }
}

impl<H> Clone for SgCbMgr<H> {
    fn clone(&self) -> SgCbMgr<H> {
        SgCbMgr {
            sender: self.sender.clone(),
            callbacks: self.callbacks.clone()
        }
    }
}
