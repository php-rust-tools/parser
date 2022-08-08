use std::{collections::{HashSet, HashMap}, hash::Hash};
use std::rc::Rc;

pub struct Gc<T> {
    sweep: usize,
    heap: HashSet<GBox<T>>,
    roots: HashMap<GBox<T>, Rc<()>>,
}

impl<T> Gc<T> {
    pub fn new() -> Self {
        Self {
            sweep: 0,
            heap: HashSet::default(),
            roots: HashMap::default(),
        }
    }

    pub fn hold(&mut self, value: T) -> GBox<T> {
        let ptr = Box::into_raw(Box::new(value));
        let gbox = GBox { ptr };

        self.heap.insert(gbox);
        gbox
    }

    pub fn get(&self, gbox: GBox<T>) -> Option<&T> {
        if self.heap.contains(&gbox) {
            Some(unsafe { &*gbox.ptr })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, gbox: GBox<T>) -> Option<&mut T> {
        if self.heap.contains(&gbox) {
            Some(unsafe { &mut *gbox.ptr })
        } else {
            None
        }
    }

    pub fn root(&mut self, value: T) -> GRoot<T> {
        let gbox = self.hold(value);
        let rc = Rc::new(());

        self.roots.insert(gbox, rc);

        GRoot { gbox }
    }

    pub fn collect(&mut self) {
        let sweep = self.sweep + 1;

        for gbox in &self.heap {
            
        }

        self.sweep = sweep;
    }
}

pub struct GRoot<T> {
    gbox: GBox<T>,
}

pub struct GBox<T> {
    ptr: *mut T,
}

impl<T> PartialEq for GBox<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Eq for GBox<T> {}

impl<T> Clone for GBox<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Copy for GBox<T> {}

impl<T> Hash for GBox<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}