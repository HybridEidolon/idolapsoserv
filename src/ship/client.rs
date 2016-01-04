use psomsg::bb::BbSecurityData;

#[derive(Clone, Default)]
pub struct ClientState {
    pub sec_data: BbSecurityData,
    pub team_id: u32,
    pub bb_guildcard: u32
}
