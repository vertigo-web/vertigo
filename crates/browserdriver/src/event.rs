use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum EventModel {
    OnClick {
        nodeId: u64
        //data: OnClickEvent            dane niesione przez event onclick  
    },
}