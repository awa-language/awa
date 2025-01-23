use std::collections::HashMap;

use super::instruction::{Handle, Value};
use ecow::EcoString;

#[derive(Debug)]
pub enum Object {
    String(EcoString),
    Slice(Vec<Value>),
    Struct {
        name: EcoString,
        fields: HashMap<EcoString, Value>,
    },
}

pub struct GC {
    pub heap: Vec<Object>,
    pub marked: Vec<bool>,
    pub alloc_count: usize,
    pub threshold: usize,
}

impl Default for GC {
    fn default() -> Self {
        Self::new()
    }
}

impl GC {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            marked: Vec::new(),
            alloc_count: 0,
            threshold: 10,
        }
    }

    pub fn allocate(&mut self, object: Object) -> Handle {
        let index = self.heap.len();
        self.heap.push(object);
        self.marked.push(false);
        self.alloc_count += 1;
        Handle(index)
    }

    pub fn get(&self, handle: Handle) -> &Object {
        &self.heap[handle.0]
    }

    pub fn get_mut(&mut self, handle: Handle) -> &mut Object {
        &mut self.heap[handle.0]
    }

    pub fn collect_garbage(
        &mut self,
        stack: &[Value],
        environments_stack: &[HashMap<EcoString, Value>],
    ) {
        for mark in &mut self.marked {
            *mark = false;
        }

        for value in stack {
            self.mark_value(value);
        }
        for env in environments_stack {
            for val in env.values() {
                self.mark_value(val);
            }
        }

        self.compact();

        self.alloc_count = 0;
    }

    fn mark_value(&mut self, value: &Value) {
        match value {
            Value::Ref(hadle) => {
                self.mark_object(*hadle);
            }
            Value::Slice(slice) => {
                for inner in slice {
                    self.mark_value(inner);
                }
            }
            Value::Struct { fields, .. } => {
                for inner in fields.values() {
                    self.mark_value(inner);
                }
            }
            _ => {}
        }
    }

    fn mark_object(&mut self, handle: Handle) {
        let mut stack = vec![handle];

        while let Some(handle) = stack.pop() {
            let idx = handle.0;
            if idx >= self.marked.len() {
                continue;
            }
            if self.marked[idx] {
                continue;
            }
            self.marked[idx] = true;

            match &self.heap[idx] {
                Object::String(_) => {}
                Object::Slice(slice) => {
                    for value in slice {
                        Self::collect_children(value, &mut stack);
                    }
                }
                Object::Struct { fields, .. } => {
                    for value in fields.values() {
                        Self::collect_children(value, &mut stack);
                    }
                }
            }
        }
    }

    fn collect_children(val: &Value, stack: &mut Vec<Handle>) {
        match val {
            Value::Ref(handle) => {
                stack.push(*handle);
            }
            Value::Slice(slice) => {
                for value in slice {
                    Self::collect_children(value, stack);
                }
            }
            Value::Struct { fields, .. } => {
                for value in fields.values() {
                    Self::collect_children(value, stack);
                }
            }
            _ => {}
        }
    }

    /// "Compaction" phase: discard all objects that are not marked as `true`,
    /// and move "alive" objects to a new vector. At the same time, update their indices (Handles).
    fn compact(&mut self) {
        let old_size = self.heap.len();

        // Create a new vector for the "surviving" objects.
        let mut new_heap = Vec::with_capacity(old_size);
        let mut new_marked = Vec::with_capacity(old_size);

        // Array for "remapping": `remap[i] = Some(j)` means
        // that object `i` from the old heap has been moved to position `j` in the new heap.
        // If `remap[i] = None`, it means object `i` is "dead".
        let mut remap = vec![None; old_size];

        // 1) Move "alive" objects to `new_heap`
        let mut new_index = 0;
        for (i, item) in remap.iter_mut().enumerate().take(old_size) {
            if self.marked[i] {
                // "Alive" object
                *item = Some(new_index);

                // Move the object to the newly created vector
                let obj = std::mem::replace(&mut self.heap[i], Object::String("".into()));
                new_heap.push(obj);

                // You can reset the mark to `false` (or directly set it to `true`, but typically it’s reset)
                new_marked.push(false);

                new_index += 1;
            } else {
                // "Dead" object
                *item = None;
            }
        }

        // 2) Traverse all "surviving" objects and fix their internal references
        for obj in &mut new_heap {
            Self::update_object_handles(obj, &remap);
        }

        // 3) Replace the old `heap` and `marked` arrays
        self.heap = new_heap;
        self.marked = new_marked;
    }
    /// Traverse the fields of the object and update the Handles to the new indices.
    fn update_object_handles(obj: &mut Object, remap: &[Option<usize>]) {
        match obj {
            Object::String(_) => {}
            Object::Slice(slice) => {
                for value in slice {
                    Self::update_value_handles(value, remap);
                }
            }
            Object::Struct { fields, .. } => {
                for value in fields.values_mut() {
                    Self::update_value_handles(value, remap);
                }
            }
        }
    }

    fn update_value_handles(val: &mut Value, remap: &[Option<usize>]) {
        match val {
            Value::Ref(handle) => {
                // Find the new index to which the old handle.0 has been moved.
                let old = handle.0;
                if let Some(new_idx) = remap[old] {
                    handle.0 = new_idx;
                } else {
                    // This means that the object was considered "alive",
                    // but inside it, there is a reference to a "dead" object —
                    // usually, this should not happen, as an "alive" object
                    // cannot contain a reference to a "dead" one.
                    // However, if this happens — either panic or ignore it.
                    panic!("Live object had reference to dead object, old index = {old}");
                }
            }
            Value::Slice(vs) => {
                for inner in vs {
                    Self::update_value_handles(inner, remap);
                }
            }
            Value::Struct { fields, .. } => {
                for inner in fields.values_mut() {
                    Self::update_value_handles(inner, remap);
                }
            }
            _ => {}
        }
    }
}
