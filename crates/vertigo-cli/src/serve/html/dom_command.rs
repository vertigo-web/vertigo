use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DomCommand {
    #[serde(rename = "create_node")]
    CreateNode {
        id: u64,
        name: String,
    },
    #[serde(rename = "create_text")]
    CreateText {
        id: u64,
        value: String,
    },
    #[serde(rename = "update_text")]
    UpdateText {
        id: u64,
        value: String,
    },
    #[serde(rename = "set_attr")]
    SetAttr {
        id: u64,
        name: String,
        value: String,
    },
    #[serde(rename = "remove_node")]
    RemoveNode {
        id: u64,
    },
    #[serde(rename = "remove_text")]
    RemoveText {
        id: u64,
    },
    #[serde(rename = "insert_before")]
    InsertBefore {
        parent: u64,
        child: u64,
        ref_id: Option<u64>,
    },
    #[serde(rename = "insert_css")]
    InsertCss {
        selector: String,
        value: String,
    },
    #[serde(rename = "create_comment")]
    CreateComment {
        id: u64,
        value: String,
    },
    #[serde(rename = "remove_comment")]
    RemoveComment {
        id: u64,
    },
    #[serde(rename = "callback_add")]
    CallbackAdd {
        id: u64,
        event_name: String,
        callback_id: u64,
    },
    #[serde(rename = "callback_remove")]
    CallbackRemove {
        id: u64,
        event_name: String,
        callback_id: u64,
    }
}


#[cfg(test)] 
mod tests {
    use super::DomCommand;

    #[test]
    fn deserialize() {

        let data = r#"
            [
                {"id":2,"name":"div","type":"create_node"},
                {"callback_id":2,"event_name":"hook_keydown","id":2,"type":"callback_add"}
            ]
        "#;

        let commands = serde_json::from_str::<Vec<DomCommand>>(data).unwrap();

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], DomCommand::CreateNode { id: 2, name: "div".into() });
        assert_eq!(commands[1], DomCommand::CallbackAdd { id: 2, event_name: "hook_keydown".into(), callback_id: 2 } );
    }

    #[test]
    fn deserialize2() {
        let data = r#"
            [
                {"id":2,"name":"div","type":"create_node"},
                {"callback_id":2,"event_name":"hook_keydown","id":2,"type":"callback_add"},
                {"id":3,"name":"ul","type":"create_node"},
                {"selector":".autocss_1","type":"insert_css","value":"list-style-type: none; margin: 10px; padding: 0"},
                {"id":3,"name":"class","type":"set_attr","value":"autocss_1"},
                {"id":4,"name":"li","type":"create_node"},
                {"selector":".autocss_2:hover","type":"insert_css","value":"text-decoration: underline;"},
                {"selector":".autocss_2","type":"insert_css","value":"display: inline; width: 60px; padding: 5px 10px; margin: 5px; cursor: pointer; background-color: lightgreen"},
                {"id":4,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":3,"event_name":"mousedown","id":4,"type":"callback_add"},
                {"id":5,"type":"create_text","value":"Counters"},
                {"child":5,"parent":4,"ref_id":null,"type":"insert_before"},
                {"child":4,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":6,"name":"li","type":"create_node"},
                {"selector":".autocss_3:hover","type":"insert_css","value":"text-decoration: underline;"},
                {"selector":".autocss_3","type":"insert_css","value":"display: inline; width: 60px; padding: 5px 10px; margin: 5px; cursor: pointer; background-color: lightblue"},
                {"id":6,"name":"class","type":"set_attr","value":"autocss_3"},
                {"callback_id":4,"event_name":"mousedown","id":6,"type":"callback_add"},
                {"id":7,"type":"create_text","value":"Animations"},
                {"child":7,"parent":6,"ref_id":null,"type":"insert_before"},
                {"child":6,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":8,"name":"li","type":"create_node"},
                {"id":8,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":5,"event_name":"mousedown","id":8,"type":"callback_add"},
                {"id":9,"type":"create_text","value":"Sudoku"},
                {"child":9,"parent":8,"ref_id":null,"type":"insert_before"},
                {"child":8,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":10,"name":"li","type":"create_node"},
                {"id":10,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":6,"event_name":"mousedown","id":10,"type":"callback_add"},
                {"id":11,"type":"create_text","value":"Input"},
                {"child":11,"parent":10,"ref_id":null,"type":"insert_before"},
                {"child":10,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":12,"name":"li","type":"create_node"},
                {"id":12,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":7,"event_name":"mousedown","id":12,"type":"callback_add"},
                {"id":13,"type":"create_text","value":"Github Explorer"},
                {"child":13,"parent":12,"ref_id":null,"type":"insert_before"},
                {"child":12,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":14,"name":"li","type":"create_node"},
                {"id":14,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":8,"event_name":"mousedown","id":14,"type":"callback_add"},
                {"id":15,"type":"create_text","value":"Game Of Life"},
                {"child":15,"parent":14,"ref_id":null,"type":"insert_before"},
                {"child":14,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":16,"name":"li","type":"create_node"},
                {"id":16,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":9,"event_name":"mousedown","id":16,"type":"callback_add"},
                {"id":17,"type":"create_text","value":"Chat"},
                {"child":17,"parent":16,"ref_id":null,"type":"insert_before"},
                {"child":16,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":18,"name":"li","type":"create_node"},
                {"id":18,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":10,"event_name":"mousedown","id":18,"type":"callback_add"},
                {"id":19,"type":"create_text","value":"Todo"},
                {"child":19,"parent":18,"ref_id":null,"type":"insert_before"},
                {"child":18,"parent":3,"ref_id":null,"type":"insert_before"},
                {"id":20,"name":"li","type":"create_node"},
                {"id":20,"name":"class","type":"set_attr","value":"autocss_2"},
                {"callback_id":11,"event_name":"mousedown","id":20,"type":"callback_add"},
                {"id":21,"type":"create_text","value":"Drop File"},
                {"child":21,"parent":20,"ref_id":null,"type":"insert_before"},
                {"child":20,"parent":3,"ref_id":null,"type":"insert_before"},
                {"child":3,"parent":2,"ref_id":null,"type":"insert_before"},
                {"id":22,"type":"create_comment","value":"value element"},
                {"id":23,"name":"div","type":"create_node"},
                {"callback_id":12,"event_name":"keydown","id":23,"type":"callback_add"},
                {"selector":".autocss_4","type":"insert_css","value":"padding: 5px"},
                {"id":23,"name":"class","type":"set_attr","value":"autocss_4"},
                {"child":2,"parent":23,"ref_id":null,"type":"insert_before"},
                {"child":22,"parent":23,"ref_id":null,"type":"insert_before"},
                {"child":22,"parent":23,"ref_id":null,"type":"insert_before"},
                {"id":24,"type":"create_comment","value":"list element"},
                {"id":25,"name":"div","type":"create_node"},
                {"id":26,"name":"div","type":"create_node"},
                {"selector":".autocss_5","type":"insert_css","value":"border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px"},
                {"id":26,"name":"class","type":"set_attr","value":"autocss_5"},
                {"callback_id":13,"event_name":"mouseenter","id":26,"type":"callback_add"},
                {"callback_id":14,"event_name":"mouseleave","id":26,"type":"callback_add"},
                {"id":27,"name":"div","type":"create_node"},
                {"selector":"@keyframes autocss_7","type":"insert_css","value":"0% { -webkit-transform: scale(0);\ntransform: scale(0); }\n100% { -webkit-transform: scale(1.0);\ntransform: scale(1.0);\nopacity: 0; }"},
                {"selector":".autocss_6","type":"insert_css","value":"width: 40px; height: 40px; background-color: #d26913; border-radius: 100%; animation: 1.0s infinite ease-in-out autocss_7 "},
                {"id":27,"name":"class","type":"set_attr","value":"autocss_6"},
                {"child":27,"parent":26,"ref_id":null,"type":"insert_before"},
                {"child":26,"parent":25,"ref_id":null,"type":"insert_before"},
                {"id":28,"name":"button","type":"create_node"},
                {"callback_id":15,"event_name":"mousedown","id":28,"type":"callback_add"},
                {"id":29,"name":"span","type":"create_node"},
                {"id":30,"type":"create_text","value":"start the progress bar"},
                {"child":30,"parent":29,"ref_id":null,"type":"insert_before"},
                {"child":29,"parent":28,"ref_id":null,"type":"insert_before"},
                {"id":31,"name":"span","type":"create_node"},
                {"child":24,"parent":31,"ref_id":null,"type":"insert_before"},
                {"child":31,"parent":28,"ref_id":null,"type":"insert_before"},
                {"child":28,"parent":25,"ref_id":null,"type":"insert_before"},
                {"child":25,"parent":23,"ref_id":22,"type":"insert_before"},
                {"child":23,"parent":1,"ref_id":null,"type":"insert_before"}
            ]
        "#;

        let commands = serde_json::from_str::<Vec<DomCommand>>(data).unwrap();

        assert_eq!(commands.len(), 97);
    }
}

