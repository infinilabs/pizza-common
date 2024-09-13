// MIT License
//
// Copyright (C) INFINI Labs & INFINI LIMITED.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::size_of;

pub struct Arena<T> {
    max_items: usize,
    max_memory_bytes: usize,
    chunks: RefCell<Vec<Vec<T>>>,
    snapshot_offsets: RefCell<Vec<(usize, usize)>>, // Stores (last_chunk_index, last_chunk_len)
    total_items: RefCell<usize>,
    total_memory_used: RefCell<usize>,
}

impl<T> fmt::Debug for Arena<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Attempt to borrow the contents of `RefCell`s
        let chunks = self.chunks.borrow();
        let snapshot_offsets = self.snapshot_offsets.borrow();
        let total_items = self.total_items.borrow();
        let total_memory_used = self.total_memory_used.borrow();

        f.debug_struct("Arena")
            .field("max_items", &self.max_items)
            .field("max_memory_bytes", &self.max_memory_bytes)
            .field("chunks", &chunks) // Debug output for the internal chunks
            .field("snapshot_offsets", &snapshot_offsets) // Debug output for snapshot offsets
            .field("total_items", &*total_items) // Dereference to get the value
            .field("total_memory_used", &*total_memory_used) // Dereference to get the value
            .finish()
    }
}

impl<T> Arena<T>
where
    T: fmt::Debug + Clone,
{
    pub fn new(initial_item_capacity: usize, max_items: usize, max_memory_bytes: usize) -> Self {
        Self {
            chunks: RefCell::new(vec![Vec::with_capacity(initial_item_capacity)]),
            snapshot_offsets: RefCell::new(Vec::new()),
            max_items,
            max_memory_bytes,
            total_items: RefCell::new(0),
            total_memory_used: RefCell::new(0),
        }
    }

    pub fn must_alloc(&self, value: T) -> &mut T {
        self.alloc(value).unwrap()
    }

    pub fn alloc(&self, value: T) -> Result<&mut T, String> {
        // Call the `alloc` method to do the allocation and return only the reference
        let (_, _, v) = self.advanced_alloc(value)?;
        Ok(v)
    }

    pub fn advanced_alloc(&self, value: T) -> Result<(usize, usize, &mut T), String> {
        let mut chunks = self.chunks.borrow_mut();
        let last_index = chunks.len() - 1;
        let element_size = size_of::<T>();

        let mut total_items = self.total_items.borrow_mut();
        let mut total_memory_used = self.total_memory_used.borrow_mut();

        if *total_items < self.max_items
            && *total_memory_used + element_size <= self.max_memory_bytes
        {
            let (chunk_index, element_index) =
                if chunks[last_index].len() < chunks[last_index].capacity() {
                    // Add to the last chunk
                    chunks[last_index].push(value);
                    (last_index, chunks[last_index].len() - 1)
                } else {
                    // Create a new chunk with double the capacity of the last chunk
                    let new_capacity = chunks[last_index].capacity() * 2;
                    let mut new_chunk = Vec::with_capacity(new_capacity);
                    new_chunk.push(value);
                    chunks.push(new_chunk);
                    let new_chunk_index = chunks.len() - 1;
                    (new_chunk_index, 0)
                };

            *total_items += 1;
            *total_memory_used += element_size;

            // Return a mutable reference to the newly pushed element along with the indices
            let chunk = &mut chunks[chunk_index];
            unsafe {
                Ok((
                    chunk_index,
                    element_index,
                    &mut *chunk.as_mut_ptr().add(element_index),
                ))
            }
        } else {
            Err(format!(
                "Arena capacity exceeded, {}/{}, {}/{}",
                *total_items, self.max_items, *total_memory_used, self.max_memory_bytes
            ))
        }
    }

    // Retrieve a reference to an element using its index
    pub fn get(&self, chunk_index: usize, element_index: usize) -> Option<core::cell::Ref<T>> {
        let chunks = self.chunks.borrow();

        // Ensure the chunk_index and element_index are within bounds
        if let Some(chunk) = chunks.get(chunk_index) {
            if let Some(item) = chunk.get(element_index) {
                // Return a Ref to the item, borrowing the entire chunk immutably
                Some(core::cell::Ref::map(chunks, |c| {
                    &c[chunk_index][element_index]
                }))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn total_chunks(&self) -> usize {
        let chunks = self.chunks.borrow();
        chunks.len()
    }

    pub fn total_items(&self) -> usize {
        *self.total_items.borrow()
    }

    pub fn total_memory_usage(&self) -> usize {
        *self.total_memory_used.borrow()
    }

    pub fn snapshot(&self) -> usize {
        let chunks = self.chunks.borrow();
        let last_chunk_index = chunks.len() - 1;
        let last_chunk_len = chunks[last_chunk_index].len();
        let mut snapshot_offsets = self.snapshot_offsets.borrow_mut();
        snapshot_offsets.push((last_chunk_index, last_chunk_len));
        snapshot_offsets.len() - 1 // Return the snapshot ID
    }

    pub fn get_snapshot(&self, snapshot: usize) -> Vec<&T> {
        let chunks = self.chunks.borrow();
        let snapshot_offsets = self.snapshot_offsets.borrow();
        let (last_chunk_index, last_chunk_len) = snapshot_offsets[snapshot];

        let mut result = Vec::new();
        for chunk in &chunks[..last_chunk_index] {
            result.extend(chunk.iter().map(|item| item as *const T));
        }
        let last_chunk = &chunks[last_chunk_index];
        result.extend(
            last_chunk
                .iter()
                .take(last_chunk_len)
                .map(|item| item as *const T),
        );

        // Unsafe block to transmute the lifetimes
        let result: Vec<&T> = unsafe { result.into_iter().map(|ptr| &*ptr).collect() };

        result
    }

    pub fn reset(&self) {
        let mut chunks = self.chunks.borrow_mut();
        chunks.clear();
        chunks.push(Vec::with_capacity(1)); // Restart with initial capacity
        *self.total_items.borrow_mut() = 0;
        *self.total_memory_used.borrow_mut() = 0;
    }
}

pub struct ArenaIterator<'a, T> {
    chunks: core::cell::Ref<'a, Vec<Vec<T>>>,
    pub batch_size: usize,
    chunk_index: usize,
    item_index: usize,
    pub next_value: Option<T>, //for VectorIterator only
}

impl<'a, T> Iterator for ArenaIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.chunk_index >= self.chunks.len() {
                return None;
            }

            let chunk = &self.chunks[self.chunk_index];

            if self.item_index < chunk.len() {
                let item = &chunk[self.item_index];
                self.item_index += 1;
                return Some(unsafe { &*(item as *const T) });
            } else {
                self.chunk_index += 1;
                self.item_index = 0;
            }
        }
    }
}

impl<T> Arena<T> {
    pub fn iter_with_batch_size(&self, batch_size: usize) -> ArenaIterator<'_, T> {
        ArenaIterator {
            chunks: self.chunks.borrow(),
            chunk_index: 0,
            item_index: 0,
            batch_size,
            next_value: None,
        }
    }

    pub fn iter(&self) -> ArenaIterator<'_, T> {
        self.iter_with_batch_size(512)
    }
}

// use crate::store::persist::PersistStore;
// use crate::store::persist::RecoverableStore;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::de::{self};
use serde::ser::SerializeStruct;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

impl<T: Serialize> Serialize for Arena<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // We need to manually serialize each field
        let mut state = serializer.serialize_struct("Arena", 6)?;
        state.serialize_field("max_items", &self.max_items)?;
        state.serialize_field("max_memory_bytes", &self.max_memory_bytes)?;
        state.serialize_field("chunks", &*self.chunks.borrow())?;
        state.serialize_field("snapshot_offsets", &*self.snapshot_offsets.borrow())?;
        state.serialize_field("total_items", &*self.total_items.borrow())?;
        state.serialize_field("total_memory_used", &*self.total_memory_used.borrow())?;
        state.end()
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arena<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            MaxItems,
            MaxMemoryBytes,
            Chunks,
            SnapshotOffsets,
            TotalItems,
            TotalMemoryUsed,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`max_items`, `max_memory_bytes`, `chunks`, `snapshot_offsets`, `total_items`, or `total_memory_used`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "max_items" => Ok(Field::MaxItems),
                            "max_memory_bytes" => Ok(Field::MaxMemoryBytes),
                            "chunks" => Ok(Field::Chunks),
                            "snapshot_offsets" => Ok(Field::SnapshotOffsets),
                            "total_items" => Ok(Field::TotalItems),
                            "total_memory_used" => Ok(Field::TotalMemoryUsed),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ArenaVisitor<T>(PhantomData<fn() -> Arena<T>>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ArenaVisitor<T> {
            type Value = Arena<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Arena")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Arena<T>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut max_items = None;
                let mut max_memory_bytes = None;
                let mut chunks = None;
                let mut snapshot_offsets = None;
                let mut total_items = None;
                let mut total_memory_used = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MaxItems => {
                            if max_items.is_some() {
                                return Err(de::Error::duplicate_field("max_items"));
                            }
                            max_items = Some(map.next_value()?);
                        }
                        Field::MaxMemoryBytes => {
                            if max_memory_bytes.is_some() {
                                return Err(de::Error::duplicate_field("max_memory_bytes"));
                            }
                            max_memory_bytes = Some(map.next_value()?);
                        }
                        Field::Chunks => {
                            if chunks.is_some() {
                                return Err(de::Error::duplicate_field("chunks"));
                            }
                            chunks = Some(map.next_value()?);
                        }
                        Field::SnapshotOffsets => {
                            if snapshot_offsets.is_some() {
                                return Err(de::Error::duplicate_field("snapshot_offsets"));
                            }
                            snapshot_offsets = Some(map.next_value()?);
                        }
                        Field::TotalItems => {
                            if total_items.is_some() {
                                return Err(de::Error::duplicate_field("total_items"));
                            }
                            total_items = Some(map.next_value()?);
                        }
                        Field::TotalMemoryUsed => {
                            if total_memory_used.is_some() {
                                return Err(de::Error::duplicate_field("total_memory_used"));
                            }
                            total_memory_used = Some(map.next_value()?);
                        }
                    }
                }

                let max_items = max_items.ok_or_else(|| de::Error::missing_field("max_items"))?;
                let max_memory_bytes =
                    max_memory_bytes.ok_or_else(|| de::Error::missing_field("max_memory_bytes"))?;
                let chunks = chunks.ok_or_else(|| de::Error::missing_field("chunks"))?;
                let snapshot_offsets =
                    snapshot_offsets.ok_or_else(|| de::Error::missing_field("snapshot_offsets"))?;
                let total_items =
                    total_items.ok_or_else(|| de::Error::missing_field("total_items"))?;
                let total_memory_used = total_memory_used
                    .ok_or_else(|| de::Error::missing_field("total_memory_used"))?;

                Ok(Arena {
                    max_items,
                    max_memory_bytes,
                    chunks: RefCell::new(chunks),
                    snapshot_offsets: RefCell::new(snapshot_offsets),
                    total_items: RefCell::new(total_items),
                    total_memory_used: RefCell::new(total_memory_used),
                })
            }
        }

        const FIELDS: &[&str] = &[
            "max_items",
            "max_memory_bytes",
            "chunks",
            "snapshot_offsets",
            "total_items",
            "total_memory_used",
        ];

        deserializer.deserialize_struct("Arena", FIELDS, ArenaVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::Range;
    use std::prelude::v1::ToString;
    use std::println;
    use std::sync::Mutex;
    use std::sync::Once;

    #[test]
    fn test_arena_allocation_and_snapshots() {
        let arena = Arena::new(4, 1000300, 1024 * 1024 * 1024);

        // Allocate some data into the arena
        arena.alloc(42).unwrap();
        arena.alloc(100).unwrap();

        // Take a snapshot
        arena.snapshot();

        println!(
            "Arena: {} {}, {}",
            arena.total_chunks(),
            arena.total_items(),
            arena.total_memory_usage()
        );

        // Allocate more data
        arena.alloc(200).unwrap();

        // Take another snapshot
        arena.snapshot();

        println!(
            "Arena: {} {}, {}",
            arena.total_chunks(),
            arena.total_items(),
            arena.total_memory_usage()
        );

        for i in 0..100 {
            arena.alloc(i).unwrap();
        }

        arena.snapshot();

        println!(
            "Arena: {} {}, {}",
            arena.total_chunks(),
            arena.total_items(),
            arena.total_memory_usage()
        );

        let snapshot_data1 = arena.get_snapshot(0);
        let snapshot_data2 = arena.get_snapshot(1);
        let snapshot_data3 = arena.get_snapshot(2);

        assert_eq!(arena.total_items(), 103);

        // Add 1M objects after snapshot, to check if any re-allocations affect the snapshot data
        for i in 0..1_000_000 {
            arena.must_alloc(i);
        }

        println!(
            "Arena: {} {}, {}",
            arena.total_chunks(),
            arena.total_items(),
            arena.total_memory_usage()
        );

        assert_eq!(arena.total_chunks(), 18);
        assert_eq!(arena.total_items(), 1_000_103);

        // Validate snapshot data
        assert_eq!(snapshot_data1, vec![&42, &100]);
        assert_eq!(snapshot_data2, vec![&42, &100, &200]);
        assert_eq!(snapshot_data3.len(), 103);
        assert_eq!(snapshot_data3[0..3], [&42, &100, &200]);

        for i in 0..100 {
            assert_eq!(snapshot_data3[3 + i], &(i as usize));
        }

        // Verify snapshots again after further allocations
        let snapshot_data1_again = arena.get_snapshot(0);
        let snapshot_data2_again = arena.get_snapshot(1);
        let snapshot_data3_again = arena.get_snapshot(2);

        assert_eq!(snapshot_data1_again, vec![&42, &100]);
        assert_eq!(snapshot_data2_again, vec![&42, &100, &200]);
        assert_eq!(snapshot_data3_again.len(), 103);

        assert_eq!(snapshot_data1, snapshot_data1_again);
        assert_eq!(snapshot_data2, snapshot_data2_again);
        assert_eq!(snapshot_data3, snapshot_data3_again);

        arena.reset();
        assert_eq!(arena.total_chunks(), 1);
        assert_eq!(arena.total_items(), 0);
        assert_eq!(arena.total_memory_usage(), 0);
    }

    #[test]
    fn test_arena_iterator() {
        let arena = Arena::new(4, 1000, 1024 * 1024 * 1024);

        for i in 0..1000 {
            arena.alloc(i).unwrap();
        }

        let mut iter = arena.iter();
        for i in 0..1000 {
            assert_eq!(iter.next(), Some(&i));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_arena_serialize_deserialize() {
        let arena = Arena::new(4, 1000, 1024 * 1024 * 1024);

        for i in 0..1000 {
            arena.alloc(i).unwrap();
        }

        // Serialize the arena
        let serialized = serde_json::to_vec(&arena).unwrap();

        // Deserialize it back to an Arena
        let deserialized: Arena<i32> = serde_json::from_slice(&serialized).unwrap();

        // Verify that the deserialized arena has the same elements
        let mut iter = deserialized.iter();
        for i in 0..1000 {
            assert_eq!(iter.next(), Some(&i));
        }
        assert_eq!(iter.next(), None);

        // Additionally, you can check if other fields are equal
        assert_eq!(arena.max_items, deserialized.max_items);
        assert_eq!(arena.max_memory_bytes, deserialized.max_memory_bytes);
        assert_eq!(
            *arena.total_items.borrow(),
            *deserialized.total_items.borrow()
        );
        assert_eq!(
            *arena.total_memory_used.borrow(),
            *deserialized.total_memory_used.borrow()
        );
    }

    #[test]
    fn test_alloc_and_retrieve() {
        let arena = Arena {
            max_items: 100,
            max_memory_bytes: 1024 * 1024, // 1 MB
            chunks: RefCell::new(vec![Vec::with_capacity(4)]),
            snapshot_offsets: RefCell::new(Vec::new()),
            total_items: RefCell::new(0),
            total_memory_used: RefCell::new(0),
        };

        let a: String = "Hello, World!".into();

        // Test alloc function
        let mut elem_ref = arena.alloc(a.clone()).expect("Allocation failed");
        assert_eq!(elem_ref, &"Hello, World!");

        let mut b = "Hello, again!".into();
        // Test advanced_alloc function to get an index
        let (chunk_index, id, elem_ref1) =
            arena.advanced_alloc(b).expect("Advanced allocation failed");

        println!("{:?},{:?}", chunk_index, id);

        // Retrieve the element using the index and verify it
        let element = arena.get(chunk_index, id).expect("Element not found");
        assert_eq!(element.as_str(), "Hello, again!");
        println!("{:?}", element);

        //update a
        elem_ref.push_str("!!!");
        let element = arena.get(0, 0).expect("Element not found");
        println!("{:?}", element);
        assert_eq!(element.as_str(), "Hello, World!!!!");

        //update b
        elem_ref1.push_str("???");
        let element = arena.get(chunk_index, id).expect("Element not found");
        assert_eq!(element.as_str(), "Hello, again!???");
        println!("{:?}", element);
    }
}
