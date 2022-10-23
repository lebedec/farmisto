use crate::Input;
use serde_json::Value;
use std::collections::hash_map::Values;
use std::collections::HashMap;

pub type Factory<B> = fn(Value) -> Box<dyn Control<B>>;

#[derive(Clone, Copy)]
pub struct RenderRect {
    position: [f32; 2],
    size: [f32; 2],
}

pub enum RenderNode<B> {
    Text {
        value: String,
        style: *const Style,
    },
    Element {
        tag: String,
        style: Style,
        children: Vec<RenderNode<B>>,
    },
    Component {
        tag: String,
        value: Value,
        controller: Box<dyn Control<B>>,
        node: Box<RenderNode<B>>,
    },
}

impl<B> RenderNode<B> {
    pub fn text(value: &str, style: *const Style) -> RenderNode<B> {
        RenderNode::Text {
            value: value.to_string(),
            style,
        }
    }

    pub fn rect(&self) -> RenderRect {
        match self {
            RenderNode::Text { style, .. } => {
                let style: &Style = unsafe { &**style };
                style.rect()
            }
            RenderNode::Element { style, .. } => style.rect(),
            RenderNode::Component { node, .. } => node.rect(),
        }
    }
}

pub struct StyleAsset {}

pub struct Style {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl Style {
    pub fn rect(&self) -> RenderRect {
        RenderRect {
            position: [self.left, self.top],
            size: [self.width, self.height],
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

fn compute_style(parent: &Style, specified: &StyleAsset) -> Style {
    unimplemented!()
}

pub struct Component<B> {
    factory: Factory<B>,
    template: String,
    stylesheet: String,
}

pub struct Interface<B> {
    pub root: RenderNode<B>,
    pub components: HashMap<String, Component<B>>,
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

    fn render(
        &mut self,
        current: &mut RenderNode<B>,
        input: &Input,
        backend: &B,
        specified: &StyleAsset,
        parent: &Style,
    ) {
        match current {
            RenderNode::Text { value, style } => {
                // once per render_template:
                // *style = parent;
            }
            RenderNode::Element {
                tag,
                children,
                style,
            } => {
                // once per render_template:
                // *style = compute_style(parent, specified);
                for node in children.iter_mut() {
                    self.render(node, input, backend, specified, style);
                }
            }
            RenderNode::Component {
                tag,
                value,
                controller,
                node,
            } => {
                controller.handle(input, node.rect());
                let next = controller.update(backend);
                let specified = StyleAsset {};
                if value != &next {
                    *node.as_mut() = self.render_template(tag, &next);
                    *value = next;
                }
                self.render(node, input, backend, &specified, parent);
            }
        }
    }

    fn render_template(&self, tag: &String, value: &Value) -> RenderNode<B> {
        let component = self.components.get(tag).unwrap();
        let template = &component.template;
        unimplemented!();
    }
}

pub trait Control<B> {
    fn create(props: Value) -> Box<dyn Control<B>>
    where
        Self: Sized;

    fn handle(&mut self, input: &Input, rect: RenderRect) {}

    fn update(&mut self, backend: &B) -> Value {
        Value::Null
    }
}

pub trait ValueArray<V> {
    fn as_array<F>(self, map: F) -> Vec<Value>
    where
        F: FnMut(&V) -> Value;
}

impl<K, V> ValueArray<V> for Values<'_, K, V> {
    fn as_array<F>(self, map: F) -> Vec<Value>
    where
        F: FnMut(&V) -> Value,
    {
        self.map(map).collect()
    }
}
