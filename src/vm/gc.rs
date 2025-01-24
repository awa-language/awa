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

        stack.into_iter().for_each(|value| {
            self.mark_value(value);
        });

        environments_stack.into_iter().for_each(|env| {
            for val in env.values() {
                self.mark_value(val);
            }
        });

        let remap = self.compact();

        for value in stack {
            Self::update_value_handles(value, &remap);
        }
        for env in environments_stack {
            for val in env.values_mut() {
                Self::update_value_handles(val, &remap);
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

    /// Пометка объекта в куче по handle, используя обход в глубину (stack-based)
    fn mark_object(&mut self, handle: Handle) {
        let mut stack = vec![handle];

        while let Some(h) = stack.pop() {
            let idx = h.0;
            if idx >= self.marked.len() {
                // Невалидный handle — пропустим
                continue;
            }
            if self.marked[idx] {
                // Уже помечен
                continue;
            }
            // Помечаем
            self.marked[idx] = true;

            // Смотрим на «детей» (вложенные ссылки) внутри объекта
            match &self.heap[idx] {
                Object::String(_) => {
                    // Не содержит ссылок
                }
                Object::Slice(elements) => {
                    for val in elements {
                        Self::collect_children(val, &mut stack);
                    }
                }
                Object::Struct { fields, .. } => {
                    for val in fields.values() {
                        Self::collect_children(val, &mut stack);
                    }
                }
            }
        }
    }

    /// Вспомогательный метод для добавления дочерних Handle в стек обхода
    fn collect_children(val: &Value, stack: &mut Vec<Handle>) {
        match val {
            Value::Ref(h) => {
                stack.push(*h);
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

    /// Сжатие кучи: отбрасываем все непомеченные объекты,
    /// «живые» объекты переезжают в новый вектор. Возвращаем карту `remap`.
    fn compact(&mut self) -> Vec<Option<usize>> {
        let old_size = self.heap.len();

        // Готовим новый вектор для "выживших" объектов
        let mut new_heap = Vec::with_capacity(old_size);
        let mut new_marked = Vec::with_capacity(old_size);

        // Массив для "переотображения" индексов
        // remap[i] = Some(j) значит объект i переехал на позицию j
        // remap[i] = None значит объект i "умер"
        let mut remap = vec![None; old_size];

        let mut new_index = 0;
        for i in 0..old_size {
            if self.marked[i] {
                // "Живой" объект
                remap[i] = Some(new_index);

                // Переносим объект i в новую кучу
                let obj = std::mem::replace(&mut self.heap[i], Object::String("".into()));
                new_heap.push(obj);

                // Метку можно сбросить (или оставить true — зависит от логики)
                new_marked.push(false);

                new_index += 1;
            } else {
                // "Мёртвый" объект
                remap[i] = None;
            }
        }

        // Проходим по всем объектам в новой куче и обновляем ссылки внутри них
        for obj in &mut new_heap {
            Self::update_object_handles(obj, &remap);
        }

        // Заменяем старую кучу новой
        self.heap = new_heap;
        self.marked = new_marked;

        remap
    }

    /// Обновить все `Handle` внутри объекта по `remap`
    fn update_object_handles(obj: &mut Object, remap: &[Option<usize>]) {
        match obj {
            Object::String(_) => {}
            Object::Slice(elements) => {
                for val in elements {
                    Self::update_value_handles(val, remap);
                }
            }
            Object::Struct { fields, .. } => {
                for val in fields.values_mut() {
                    Self::update_value_handles(val, remap);
                }
            }
        }
    }

    /// Рекурсивно обновить все `Value::Ref` (и вложенные), используя `remap`
    fn update_value_handles(val: &mut Value, remap: &[Option<usize>]) {
        match val {
            Value::Ref(handle) => {
                let old = handle.0;
                // Находим новый индекс
                if let Some(new_idx) = remap[old] {
                    handle.0 = new_idx;
                } else {
                    // Ситуация, когда "живое" значение ссылается на "умерший" объект,
                    // обычно не должна происходить при корректном mark-фазе.
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
