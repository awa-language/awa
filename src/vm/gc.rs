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
        global_vars: &HashMap<EcoString, Value>,
    ) {
        for mark in &mut self.marked {
            *mark = false;
        }

        for v in stack {
            self.mark_value(v);
        }
        for env in environments_stack {
            for val in env.values() {
                self.mark_value(val);
            }
        }
        for val in global_vars.values() {
            self.mark_value(val);
        }

        self.compact();

        self.alloc_count = 0;
    }

    fn mark_value(&mut self, value: &Value) {
        match value {
            Value::Ref(h) => {
                self.mark_object(*h);
            }
            Value::Slice(vs) => {
                for inner in vs {
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

        while let Some(h) = stack.pop() {
            let idx = h.0;
            if idx >= self.marked.len() {
                continue;
            }
            if self.marked[idx] {
                continue;
            }
            self.marked[idx] = true;

            match &self.heap[idx] {
                Object::String(_) => {}
                Object::Slice(vs) => {
                    for v in vs {
                        self.collect_children(v, &mut stack);
                    }
                }
                Object::Struct { fields, .. } => {
                    for v in fields.values() {
                        self.collect_children(v, &mut stack);
                    }
                }
            }
        }
    }

    fn collect_children(&self, val: &Value, stack: &mut Vec<Handle>) {
        match val {
            Value::Ref(h) => {
                stack.push(*h);
            }
            Value::Slice(inner) => {
                for v in inner {
                    self.collect_children(v, stack);
                }
            }
            Value::Struct { fields, .. } => {
                for v in fields.values() {
                    self.collect_children(v, stack);
                }
            }
            _ => {}
        }
    }

    /// «Компактирующий» этап: выкидываем все объекты, которые не marked = true,
    /// а "живые" объекты переносим в новый вектор. При этом меняем их индексы (Handle).
    fn compact(&mut self) {
        let old_size = self.heap.len();

        // Создадим новый вектор для «выживших» объектов.
        let mut new_heap = Vec::with_capacity(old_size);
        let mut new_marked = Vec::with_capacity(old_size);

        // Массив для «ремапинга»: `remap[i] = Some(j)` означает,
        // что объект i из старого heap переехал на позицию j в новом heap.
        // Если remap[i] = None, значит объект i — «мёртвый».
        let mut remap = vec![None; old_size];

        // 1) перенесём «живые» объекты в new_heap
        let mut new_index = 0;
        for i in 0..old_size {
            if self.marked[i] {
                // «живой» объект
                remap[i] = Some(new_index);
                // Переносим объект во вновь созданный вектор
                let obj = std::mem::replace(&mut self.heap[i], Object::String("".into()));
                new_heap.push(obj);
                // Можно поставить пометку в false (или сразу true, но обычно сбрасываем)
                new_marked.push(false);
                new_index += 1;
            } else {
                // «мёртвый» объект
                remap[i] = None;
            }
        }

        // 2) Пройдёмся по всем «выжившим» объектам и починим их внутренние ссылки
        for obj in &mut new_heap {
            self.update_object_handles(obj, &remap);
        }

        // 3) Заменим старые heap/marked
        self.heap = new_heap;
        self.marked = new_marked;
    }
    /// Обход полей объекта и обновление Handle на новые индексы
    fn update_object_handles(&self, obj: &mut Object, remap: &[Option<usize>]) {
        match obj {
            Object::String(_) => {}
            Object::Slice(vs) => {
                for v in vs {
                    self.update_value_handles(v, remap);
                }
            }
            Object::Struct { fields, .. } => {
                for v in fields.values_mut() {
                    self.update_value_handles(v, remap);
                }
            }
        }
    }

    fn update_value_handles(&self, val: &mut Value, remap: &[Option<usize>]) {
        match val {
            Value::Ref(h) => {
                // Найдём, на какой индекс переехал старый h.0
                let old = h.0;
                if let Some(new_idx) = remap[old] {
                    h.0 = new_idx;
                } else {
                    // Это означает, что объект считался «живым»,
                    // но внутри него ссылка на «мёртвый» объект —
                    // обычно такое не должно случиться, т.к. «живой» объект
                    // не может содержать Ref на «мёртвый».
                    // Но если такое случилось — либо паника, либо игнорируем.
                    panic!(
                        "Live object had reference to dead object, old index = {old}"
                    );
                }
            }
            Value::Slice(vs) => {
                for inner in vs {
                    self.update_value_handles(inner, remap);
                }
            }
            Value::Struct { fields, .. } => {
                for inner in fields.values_mut() {
                    self.update_value_handles(inner, remap);
                }
            }
            _ => {}
        }
    }
}
