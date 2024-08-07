use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Component {
    pub id: u32,
    pub name: String,
    pub desc: String,
    pub required: bool,
}

pub fn get_component_list_from_manifest() -> Vec<Component> {
    vec![
        Component {
            id: 1,
            name: "组件1".into(),
            desc: "组件1描述".into(),
            required: true,
        },
        Component {
            id: 2,
            name: "组件2".into(),
            desc: "组件2描述".into(),
            required: false,
        },
        Component {
            id: 3,
            name: "组件3".into(),
            desc: "组件3描述".into(),
            required: false,
        },
    ]
    // TODO: get_component_list_from_manifest
}
