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
        stack: &mut [Value],
        environments_stack: &mut [HashMap<EcoString, Value>],
    ) {
        for mark in &mut self.marked {
            *mark = false;
        }

        for value in stack.iter_mut() {
            self.mark_value(value);
        }

        for environment in environments_stack.iter_mut() {
            for value in environment.values() {
                self.mark_value(value);
            }
        }

        let remap = self.compact();

        for value in stack {
            Self::update_value_handles(value, &remap);
        }
        for environment in environments_stack {
            for value in environment.values_mut() {
                Self::update_value_handles(value, &remap);
            }
        }

        self.alloc_count = 0;
    }

    fn mark_value(&mut self, value: &Value) {
        match value {
            Value::Ref(handle) => {
                self.mark_object(*handle);
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
            let index = handle.0;
            if index >= self.marked.len() {
                continue;
            }
            if self.marked[index] {
                continue;
            }
            self.marked[index] = true;

            match &self.heap[index] {
                Object::String(_) => {
                }
                Object::Slice(elements) => {
                    for value in elements {
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
                for inner in slice {
                    Self::collect_children(inner, stack);
                }
            }
            Value::Struct { fields, .. } => {
                for inner in fields.values() {
                    Self::collect_children(inner, stack);
                }
            }
            _ => {}
        }
    }

    fn compact(&mut self) -> Vec<Option<usize>> {
        let old_size = self.heap.len();

        let mut new_heap = Vec::with_capacity(old_size);
        let mut new_marked = Vec::with_capacity(old_size);

        let mut remap = vec![None; old_size];

        let mut new_index = 0;
        for (i, item) in remap.iter_mut().enumerate().take(old_size) {
            if self.marked[i] {
                *item = Some(new_index);

                let object = std::mem::replace(&mut self.heap[i], Object::String("".into()));
                new_heap.push(object);

                new_marked.push(false);

                new_index += 1;
            } else {
                *item = None;
            }
        }

        for object in &mut new_heap {
            Self::update_object_handles(object, &remap);
        }

        self.heap = new_heap;
        self.marked = new_marked;

        remap
    }

    fn update_object_handles(obj: &mut Object, remap: &[Option<usize>]) {
        match obj {
            Object::String(_) => {}
            Object::Slice(elements) => {
                for value in elements {
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
                let old = handle.0;
                if let Some(new_idx) = remap[old] {
                    handle.0 = new_idx;
                } else {
                    panic!("Live object had reference to dead object, old index = {old}");
                }
            }
            Value::Slice(slice) => {
                for inner in slice {
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
