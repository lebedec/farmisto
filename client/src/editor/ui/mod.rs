use crate::bumaga::{Control, Interface, RenderNode, RenderRect, Style, ValueArray};
use crate::editor::Editor;
use crate::Input;
use game::model::TreeId;
use serde_json::{json, Value};
use std::collections::HashMap;

fn test_main(editor: &Editor, input: &Input) {
    let mut interface = Interface {
        root: RenderNode::text("Hello World!", &Style::default() as *const _),
        components: Default::default(),
    };
    interface.register("Parent", Parent::create, "parent.html", "parent.css");
    interface.register("Child", Child::create, "child.html", "child.css");
    // interface.render(editor, input);
}

pub struct Parent {}

impl Control<Editor> for Parent {
    fn create(props: Value) -> Box<dyn Control<Editor>> {
        Box::new(Self {})
    }

    fn handle(&mut self, input: &Input, rect: RenderRect) {
        // if input.click() && rect.contains(input.mouse_position()) {
        //
        // }
    }

    fn update(&mut self, editor: &Editor) -> Value {
        let items = editor.gameplay.trees.values().as_array(|tree| {
            let id: usize = tree.id.into();
            json!({
                "id": id,
                "name": &tree.kind.name
            })
        });

        json!({
            "mode": editor.active,
            "items": items
        })
    }
}

#[derive(serde::Deserialize)]
pub struct ChildProps {
    id: usize,
    name: String,
}

#[derive(serde::Serialize)]
pub struct Child {
    id: usize,
    name: String,
}

impl Control<Editor> for Child {
    fn create(props: Value) -> Box<dyn Control<Editor>> {
        let props: ChildProps = serde_json::from_value(props).unwrap();
        Box::new(Child {
            id: props.id,
            name: props.name,
        })
    }

    fn update(&mut self, editor: &Editor) -> Value {
        serde_json::to_value(self).unwrap()
        // json!({
        //     "name": self.name,
        //     "id": self.id
        // })
    }
}
