
#[derive(Clone)]
pub struct RealNodeId {
    id: u64,
}

impl RealNodeId {
    pub fn root() -> RealNodeId {
        RealNodeId {
            id: 1
        }
    }
}