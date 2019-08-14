pub trait Socket {}

pub type OwnedSockets = (
    Socket0,
    Socket1,
    Socket2,
    Socket3,
    Socket4,
    Socket5,
    Socket6,
    Socket7,
);
pub type Sockets<'a> = (
    &'a mut Socket0,
    &'a mut Socket1,
    &'a mut Socket2,
    &'a mut Socket3,
    &'a mut Socket4,
    &'a mut Socket5,
    &'a mut Socket6,
    &'a mut Socket7,
);

pub struct Socket0 {}
impl Socket for Socket0 {}
pub struct Socket1 {}
impl Socket for Socket1 {}
pub struct Socket2 {}
impl Socket for Socket2 {}
pub struct Socket3 {}
impl Socket for Socket3 {}
pub struct Socket4 {}
impl Socket for Socket4 {}
pub struct Socket5 {}
impl Socket for Socket5 {}
pub struct Socket6 {}
impl Socket for Socket6 {}
pub struct Socket7 {}
impl Socket for Socket7 {}
