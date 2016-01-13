use psomsg::bb::BbSecurityData;
use psomsg::bb::BbFullCharData;

#[derive(Clone, Default)]
pub struct ClientState {
    pub sec_data: BbSecurityData,
    pub account_id: u32,
    pub team_id: u32,
    pub bb_guildcard: u32,
    pub full_char: Option<BbFullCharData>,
    pub connection_id: usize
}
