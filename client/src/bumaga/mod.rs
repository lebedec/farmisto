use crate::editor::Editor;
use crate::Input;
use game::model::TreeId;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

pub type Factory<B> = fn(Value) -> Box<dyn Control<B>>;

pub struct Component<B> {
    factory: Factory<B>,
    template: String,
    stylesheet: String,
}

pub struct Interface<B> {
    root: RenderObject<B>,
    components: HashMap<String, Component<B>>,
}

impl<B> Interface<B> {
    pub fn register(&mut self, tag: &str, factory: Factory<B>, template: &str, stylesheet: &str) {
        self.components.insert(
            tag.to_string(),
            Component {
                factory,
                template: template.to_string(),
                stylesheet: stylesheet.to_string(),
            },
        );
    }

    pub fn render(&mut self, backend: &B, input: &Input) {
        // render tree
        match &self.root {
            RenderObject::Text(_) => {}
            RenderObject::Element { .. } => {}
            RenderObject::Component { controller, tag } => {
                let value = controller.render(backend);
                let component = self.components.get(tag).unwrap();
                let element = value_to_element::<B>(component.template.clone(), value);
                let controller = (component.factory)(Value::Null);
                self.root = RenderObject::Component {
                    controller,
                    tag: "a".to_string(),
                }
            }
        }
        // calculate styles
        // ..

        // handle input

        // calculate bounds only (fixed position + resolved length)
        // :active
    }
}

fn value_to_element<B>(template: String, value: Value) -> RenderObject<B> {
    unimplemented!()
}

fn test_main(editor: &Editor, input: &Input) {
    let mut interface = Interface {
        root: RenderObject::Text("".to_string()),
        components: Default::default(),
    };
    interface.register("Parent", Parent::create, "parent.html", "parent.css");
    interface.register("Child", Child::create, "child.html", "child.css");
    interface.render(editor, input);
}

pub trait Control<B> {
    fn boxed(self) -> Box<dyn Control<B>>
    where
        Self: 'static + Sized,
    {
        Box::new(self)
    }

    fn create(props: Value) -> Box<dyn Control<B>>
    where
        Self: Sized;

    fn handle(&self, backend: &mut B, input: &Input) {}

    // fn template(&self) -> String;

    fn render(&self, backend: &B) -> Value {
        Value::Null
    }
}

pub enum RenderObject<B> {
    Text(String),
    Element {
        children: Vec<RenderObject<B>>,
    },
    Component {
        controller: Box<dyn Control<B>>,
        tag: String,
    },
}

pub struct Parent {}

impl Control<Editor> for Parent {
    fn create(props: Value) -> Box<dyn Control<Editor>> {
        Box::new(Self {})
    }

    fn handle(&self, editor: &mut Editor, input: &Input) {
        // render properties
        if input.click() {
            editor.do_something();
        }
    }

    // fn template(&self) -> String {
    //     r#"
    //     <div>
    //         <h1>Editor: {{mode}}</h1>
    //         {{#items}}
    //         <Child key="{{id}}" id="{{id}}" name="{{name}}">
    //         {{/items}}
    //     </div>
    //     "#
    //     .to_string()
    // }

    fn render(&self, editor: &Editor) -> Value {
        let items: Vec<Value> = editor
            .gameplay
            .trees
            .values()
            .map(|tree| {
                let id: usize = tree.id.into();
                json!({ "id": id, "name": &tree.kind.name })
            })
            .collect();
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

pub struct Child {
    id: TreeId,
    name: String,
}

impl Control<Editor> for Child {
    fn create(props: Value) -> Box<dyn Control<Editor>> {
        let props: ChildProps = serde_json::from_value(props).unwrap();
        Box::new(Child {
            id: props.id.into(),
            name: props.name,
        })
    }

    // fn template(&self) -> String {
    //     r#"
    //     <h2>{{name}} : {{x}}</h2>
    //     "#
    //     .to_string()
    // }

    fn render(&self, editor: &Editor) -> Value {
        let tree = editor.gameplay.trees.get(&self.id).unwrap();
        json!({
            "name": self.name,
            "x": tree.position.x
        })
    }
}
