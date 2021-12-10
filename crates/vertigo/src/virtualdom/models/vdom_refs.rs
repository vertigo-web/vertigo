use std::collections::HashMap;
use std::fmt;

use crate::Driver;
use super::realdom_id::RealDomId;

#[derive(Clone)]
pub struct NodeRefsItem {
    id: RealDomId,
    driver: Driver,
}

impl fmt::Debug for NodeRefsItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRefsItem")
            .field("item", &"Rc<NodeRefsItemTrait>")
            .finish()
    }
}

impl NodeRefsItem {
    pub fn new(driver: Driver, id: RealDomId) -> NodeRefsItem {
        NodeRefsItem {
            id,
            driver,
        }
    }

    pub fn get_bounding_client_rect_x(&self) -> f64 {
        self.driver.get_bounding_client_rect_x(self.id)
    }

    pub fn get_bounding_client_rect_y(&self) -> f64 {
        self.driver.get_bounding_client_rect_y(self.id)
    }

    pub fn get_bounding_client_rect_width(&self) -> f64 {
        self.driver.get_bounding_client_rect_width(self.id)
    }

    pub fn get_bounding_client_rect_height(&self) -> f64 {
        self.driver.get_bounding_client_rect_height(self.id)
    }

    pub fn scroll_top(&self) -> i32 {
        self.driver.scroll_top(self.id)
    }

    pub fn set_scroll_top(&self, value: i32) {
        self.driver.set_scroll_top(self.id, value);
    }

    pub fn scroll_left(&self) -> i32 {
        self.driver.scroll_left(self.id)
    }

    pub fn set_scroll_left(&self, value: i32) {
        self.driver.set_scroll_left(self.id, value);
    }

    pub fn scroll_width(&self) -> i32 {
        self.driver.scroll_width(self.id)
    }

    pub fn scroll_height(&self) -> i32 {
        self.driver.scroll_height(self.id)
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

        &[]
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
