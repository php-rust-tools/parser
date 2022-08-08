use std::{collections::{HashSet, HashMap}, hash::Hash};
use std::rc::Rc;

pub struct Gc<T: Trace<T>> {
    sweep: usize,
    heap: HashSet<GBox<T>>,
    roots: HashMap<GBox<T>, Rc<()>>,
}

impl<T: Trace<T>> Gc<T> {
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

        let mut tracer = Tracer { sweep, heap: &self.heap };

        for gbox in &self.heap {
            tracer.mark(*gbox);
            // 1. Mark the object.
            // 2. Trace the object.
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

pub trait Trace<T> {
    fn trace();
}

pub struct Tracer<'t, T: Trace<T>> {
    sweep: usize,
    heap: &'t HashSet<GBox<T>>,
}

impl<'t, T: Trace<T>> Tracer<'t, T> {
    pub fn mark(&mut self, gbox: GBox<T>) {
        // 1. Has the provided GBox already been swept?
        // 2. If it has, and the current sweep is not 
        //    equal to the last sweep the GBox was found in,
        //    trace the T value inside of the GBox.
    }
}