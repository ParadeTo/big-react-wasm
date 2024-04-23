#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkTag {
    FunctionComponent = 0,
    HostRoot = 3,
    HostComponent = 5,
    HostText = 6,
}
