use std::{collections::{HashMap}, rc::Rc};
use std::fmt;
pub trait NodeRefsItemTrait {
    fn get_bounding_client_rect(&self) -> (f64, f64, f64, f64);     //x, y, width, height
    fn scroll_top(&self) -> i32;
    fn set_scroll_top(&self, value: i32);
    fn scroll_left(&self) -> i32;
    fn set_scroll_left(&self, value: i32);
    fn scroll_width(&self) -> i32;
    fn scroll_height(&self) -> i32;
}

#[derive(Debug)]
pub struct NodeRefsItemRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone)]
pub struct NodeRefsItem {
    item: Rc<dyn NodeRefsItemTrait>,
}

impl fmt::Debug for NodeRefsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRefsItem")
            .field("item", &"Rc<NodeRefsItemTrait>")
            .finish()
    }
}

impl NodeRefsItem {
    pub fn new<I: NodeRefsItemTrait + 'static>(item: I) -> NodeRefsItem {
        NodeRefsItem {
            item: Rc::new(item)
        }
    }

    pub fn get_bounding_client_rect(&self) -> NodeRefsItemRect {
        let (x, y, width, height) = self.item.get_bounding_client_rect();
        NodeRefsItemRect { x, y, width, height }
    }

    pub fn scroll_top(&self) -> i32 {
        self.item.scroll_top()
    }

    pub fn set_scroll_top(&self, value: i32) {
        self.item.set_scroll_top(value);
    }

    pub fn scroll_left(&self) -> i32 {
        self.item.scroll_left()
    }

    pub fn set_scroll_left(&self, value: i32) {
        self.item.set_scroll_left(value);
    }

    pub fn scroll_width(&self) -> i32 {
        self.item.scroll_width()
    }

    pub fn scroll_height(&self) -> i32 {
        self.item.scroll_height()
    }
}

pub struct NodeRefs {
    data: HashMap<&'static str, Vec<NodeRefsItem>>
}

impl NodeRefs {
    pub(crate) fn new() -> NodeRefs {
        NodeRefs {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, ref_name: &str) -> &[NodeRefsItem] {
        let item = self.data.get(ref_name);

        if let Some(item) = item {
            let item = item.as_slice();
            return item;
        }

        return &[];
    }

    pub fn expect_one(&self, ref_name: &str) -> Option<NodeRefsItem> {
        if let [item] = self.get(ref_name) {
            return Some(item.clone());
        }

        None
    }

    pub(crate) fn set(&mut self, ref_name: &'static str, item: NodeRefsItem) {
        let list = self.data.entry(ref_name).or_insert_with(Vec::new);
        list.push(item);
    }
}
