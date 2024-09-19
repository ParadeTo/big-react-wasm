#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WorkTag {
    FunctionComponent = 0,
    HostRoot = 3,
    HostComponent = 5,
    HostText = 6,
    Fragment = 7,
    ContextProvider = 8,
    SuspenseComponent = 13,
    OffscreenComponent = 14,
    MemoComponent = 15,
    LazyComponent = 16,
}
