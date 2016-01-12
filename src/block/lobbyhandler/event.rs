
/// Seasonal events
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Event {
    Normal = 0,
    Christmas = 1,
    Valentines = 3,
    Easter = 4,
    Halloween = 5,
    Sonic = 6,
    NewYears = 7,
    Spring = 8,
    WhiteDay = 9,
    Wedding = 10,
    Autumn = 11,
    Flags = 12,
    SpringFlag = 13,
    AltNormal = 14
}
