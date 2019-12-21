// vim: set ai et ts=4 sts=4 sw=4:
#![allow(unused)]

use std::rc::{Rc, Weak};
use std::clone::Clone;
use std::cell::{RefCell, Ref, RefMut};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::fmt;

pub enum VisitResult {
    Accept,
    Reject,
}

#[derive(Debug)]
struct Node<T> {
    parent: Option<Weak<RefCell<Node<T>>>>,
    children: Vec<Rc<RefCell<Node<T>>>>,
    data: T,
}

#[derive(Debug)]
pub struct NodeRef<T>(Rc<RefCell<Node<T>>>);

impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        // creates a separate reference to the same contained node (does not actually copy any data)
        NodeRef(self.0.clone())
    }
}

impl<T: Clone> NodeRef<T> {
    pub fn clone_tree(&self) -> NodeRef<T> {
        let new_node_rc: Rc<RefCell<Node<T>>> = Rc::new(RefCell::new(Node {
            parent: None,
            children: vec![],
            data: self.0.borrow().data.clone(),
        }));
        for rc in &self.0.borrow().children {
            let cloned_child: NodeRef<T> = NodeRef(rc.clone()).clone_tree();
            cloned_child.0.borrow_mut().parent = Some(Rc::downgrade(&new_node_rc));
            new_node_rc.borrow_mut().children.push(cloned_child.0);
        }
        NodeRef(new_node_rc)
    }
}
impl<T> PartialEq for NodeRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for NodeRef<T> {}

impl<T> Hash for NodeRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // hash by address that the wrapped Rc is pointing to
        let rc: &Rc<RefCell<Node<T>>> = &self.0;
        ptr::hash(&**rc, state); // rc = &Rc, *rc = Rc, **rc = contained obj, &**rc = addr of contained obj
    }
}

impl<T> NodeRef<T> {
    pub fn new(data: T) -> NodeRef<T> {
        NodeRef(Rc::new(RefCell::new(Node {
            parent: None,
            children: vec![],
            data: data,
        })))
    }

    pub fn parent(&self) -> Option<NodeRef<T>> {
        match self.0.borrow().parent.as_ref() {
            None     => None,
            Some(wk) => match wk.upgrade() {
                None     => None,
                Some(rc) => Some(NodeRef(Rc::clone(&rc))),
            }
        }
    }

    pub fn borrow_data(&self) -> Ref<T> {
        Ref::map(self.0.borrow(), |nd| &nd.data)
    }
    pub fn borrow_data_mut(&self) -> RefMut<T> {
        RefMut::map(self.0.borrow_mut(), |nd| &mut nd.data)
    }

    pub fn children(&self) -> Children<T> {
        Children::new(&self)
    }
    pub fn num_children(&self) -> usize {
        self.0.borrow().children.len()
    }
    pub fn descendants(&self) -> Descendants<T> {
        Descendants::new(&self)
    }
    pub fn visit_descendants<C>(&self, mut callback: C)
        where C: FnMut(&Self) -> VisitResult
    {
        self.visit_descendants_r(&mut callback);
    }
    fn visit_descendants_r<C>(&self, callback: &mut C)
        where C: FnMut(&Self) -> VisitResult
    {
        let result_self = callback(&self);
        if let VisitResult::Accept = result_self {
            for child in self.children() {
                child.visit_descendants_r(callback);
            }
        }
    }
    pub fn ancestors(&self) -> Ancestors<T> {
        Ancestors::new(&self)
    }
    pub fn is_leaf(&self) -> bool {
        self.num_children() == 0
    }

    pub fn add_child(&self, node: &NodeRef<T>) {
        node.0.borrow_mut().parent = Some(Rc::downgrade(&self.0));
        self.0.borrow_mut().children.push(node.0.clone());
    }

    pub fn remove_child(&self, node: &NodeRef<T>) -> bool {
        let child_idx = self.0.borrow().children.iter().position(|c| Rc::ptr_eq(c, &node.0));
        if let Some(idx) = child_idx {
            self.0.borrow_mut().children.remove(idx);
            return true;
        }
        //if let Some(child_idx) = self.0.borrow().children.iter().position(|c| Rc::ptr_eq(c, &node.0)) {
        //    self.0.borrow_mut().children.remove(child_idx);
        //    return true;
        //}
        return false;
    }
}

pub struct Descendants<T> {
    node: NodeRef<T>,
    counter: usize,
    child_iterators: Vec<Descendants<T>>,
}
impl<T> Descendants<T> {
    pub fn new(of: &NodeRef<T>) -> Self {
        Self {
            node: of.clone(),
            counter: 0,
            child_iterators: vec![],
        }
    }
}
impl<T> Iterator for Descendants<T> {
    type Item = NodeRef<T>;
    fn next(&mut self) -> Option<NodeRef<T>> {
        if self.counter == 0 {
            self.counter += 1;
            self.child_iterators = self.node.0.borrow().children
                                       .iter()
                                       .map(|c| Descendants::new(&NodeRef(c.clone())))
                                       .collect();
            return Some(self.node.clone());
        }
        let child_idx = self.counter - 1;
        if child_idx >= self.child_iterators.len() {
            return None;
        }
        match self.child_iterators[child_idx].next() {
            Some(c) => Some(c),
            None    => { self.counter += 1; self.next() }
        }
    }
}

pub struct Ancestors<T> {
    node: Option<NodeRef<T>>,
}
impl<T> Ancestors<T> {
    pub fn new(of: &NodeRef<T>) -> Self {
        Self {
            node: Some(of.clone()),
        }
    }
}
impl<T> Iterator for Ancestors<T> {
    type Item = NodeRef<T>;
    fn next(&mut self) -> Option<NodeRef<T>> {
        match self.node.as_ref().unwrap().parent() {
            None    => None,
            Some(p) => {
                self.node = Some(p.clone());
                Some(p)
            },
        }
    }
}

pub struct Children<T> {
    node: NodeRef<T>,
    counter: usize,
}
impl<T> Children<T> {
    pub fn new(of: &NodeRef<T>) -> Self {
        Self {
            node: of.clone(),
            counter: 0,
        }
    }
}
impl<T> Iterator for Children<T> {
    type Item = NodeRef<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let node: Ref<Node<T>> = self.node.0.borrow();
        if self.counter >= node.children.len() {
            return None;
        }
        self.counter += 1;
        Some(NodeRef(node.children[self.counter-1].clone()))
    }
}

impl<T: fmt::Display> fmt::Display for NodeRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        fn fmt_r<T: fmt::Display>(node: &NodeRef<T>, depth: usize, s: &mut String) {
            s.push_str(&format!("{}{}\n", " ".repeat(depth*4), *node.borrow_data()));
            for rc in &node.0.borrow().children {
                fmt_r(&NodeRef(rc.clone()), depth+1, s);
            }
        }

        let mut s: String = String::new();
        fmt_r(&self, 0, &mut s);
        s.truncate(s.trim_end().len()); // trim in place
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[derive(Debug,Clone)]
    struct DummyData {
        int: i32,
    }

    #[test]
    fn eq_hash_clone() {
        macro_rules! hash_of {
            ($thing:expr) => {{
                let mut hasher = DefaultHasher::new();
                $thing.hash(&mut hasher);
                hasher.finish()
            }}
        }
        let node = NodeRef::new(DummyData { int: 13 });
        let clone = node.clone();
        // clone contains another Rc to the same node data, so should compare and hash the same
        assert_eq!(node, clone);
        assert_eq!(hash_of!(node), hash_of!(clone));

        // contains the same data, but is physically a different node instance, so shouldn't compare the same
        let similar_node = NodeRef::new(DummyData { int: 13 });
        assert_ne!(similar_node, node);
        assert_ne!(hash_of!(similar_node), hash_of!(node));

        // same thing for clone_tree
        let cloned_tree = node.clone_tree();
        assert_ne!(cloned_tree, node);
        assert_ne!(hash_of!(cloned_tree), hash_of!(node));
    }
}
