use std::collections::HashMap;

use super::instruction::{Handle, Value};
use ecow::EcoString;

#[derive(Debug)]
pub enum Object {
    String(EcoString),
    Array(Vec<Value>),
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
    object_pool: ObjectPool,
    mark_stack: Vec<Handle>,
}

struct ObjectPool {
    strings: Vec<EcoString>,
    slices: Vec<Vec<Value>>,
    structs: Vec<HashMap<EcoString, Value>>,
}

impl ObjectPool {
    fn new() -> Self {
        Self {
            strings: Vec::with_capacity(100),
            slices: Vec::with_capacity(100),
            structs: Vec::with_capacity(100),
        }
    }

    fn get_string(&mut self) -> EcoString {
        self.strings
            .pop()
            .unwrap_or_else(|| EcoString::with_capacity(10))
    }

    fn get_slice(&mut self) -> Vec<Value> {
        self.slices.pop().unwrap_or_else(|| Vec::with_capacity(10))
    }

    fn get_struct(&mut self) -> HashMap<EcoString, Value> {
        self.structs
            .pop()
            .unwrap_or_else(|| HashMap::with_capacity(5))
    }
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
            object_pool: ObjectPool::new(),
            mark_stack: Vec::new(),
        }
    }

    pub fn allocate(&mut self, object: Object) -> Handle {
        let index = self.heap.len();
        if index == usize::MAX {
            panic!("GC heap overflow");
        }

        let reused_object = match object {
            Object::String(string) => {
                let mut pooled = self.object_pool.get_string();
                pooled.clear();
                pooled.push_str(&string);

                Object::String(pooled)
            }
            Object::Array(value) => {
                let mut pooled = self.object_pool.get_slice();
                pooled.clear();
                pooled.extend(value);

                Object::Array(pooled)
            }
            Object::Struct { name, fields } => {
                let mut pooled = self.object_pool.get_struct();
                pooled.clear();
                pooled.extend(fields);
                Object::Struct {
                    name,
                    fields: pooled,
                }
            }
        };

        self.heap.push(reused_object);
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
        self.threshold += self.threshold / 2;

        self.marked.clear();
        self.marked.resize(self.heap.len(), false);

        for value in stack.iter() {
            self.mark_value(value);
        }

        for environment in environments_stack.iter() {
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
        self.mark_stack.clear();
        self.mark_stack.push(handle);

        while let Some(handle) = self.mark_stack.pop() {
            let index = handle.0;
            if index >= self.marked.len() || self.marked[index] {
                continue;
            }

            self.marked[index] = true;

            match &self.heap[index] {
                Object::String(_) => {}
                Object::Array(elements) => {
                    for value in elements {
                        Self::collect_children(value, &mut self.mark_stack);
                    }
                }
                Object::Struct { fields, .. } => {
                    for value in fields.values() {
                        Self::collect_children(value, &mut self.mark_stack);
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
        let marked_count = self.marked.iter().filter(|&&m| m).count();
        let mut new_heap = Vec::with_capacity(marked_count);
        let mut remap = vec![None; self.heap.len()];

        for (i, marked) in self.marked.iter().enumerate() {
            if *marked {
                remap[i] = Some(new_heap.len());
                let mut object =
                    std::mem::replace(&mut self.heap[i], Object::Array(Vec::with_capacity(100)));

                match &mut object {
                    Object::String(s) if s.is_empty() => {
                        self.object_pool.strings.push(std::mem::take(s));
                    }
                    Object::Array(v) if v.is_empty() => {
                        self.object_pool.slices.push(std::mem::take(v));
                    }
                    Object::Struct { fields, .. } if fields.is_empty() => {
                        self.object_pool.structs.push(std::mem::take(fields));
                    }
                    _ => {}
                }

                new_heap.push(object);
            }
        }

        for object in &mut new_heap {
            Self::update_object_handles(object, &remap);
        }

        self.heap = new_heap;
        remap
    }

    fn update_object_handles(obj: &mut Object, remap: &[Option<usize>]) {
        match obj {
            Object::String(_) => {}
            Object::Array(elements) => {
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
