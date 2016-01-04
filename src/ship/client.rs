use psomsg::bb::BbSecurityData;

#[derive(Clone, Default)]
pub struct ClientState {
    pub sec_data: BbSecurityData
}
