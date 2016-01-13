use std::net::SocketAddrV4;

use psomsg::bb::BbSecurityData;

#[derive(Clone, Default)]
pub struct ClientState {
    pub sec_data: BbSecurityData,
    pub team_id: u32,
    pub bb_guildcard: u32,
    pub ships: Option<Vec<(SocketAddrV4, String)>>,
    pub options: u32,
    pub key_config: Vec<u8>,
    pub joy_config: Vec<u8>
}
