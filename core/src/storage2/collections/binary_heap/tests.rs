// Copyright 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::BinaryHeap;
use crate::{
    env,
    storage2::traits::{
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
};
use ink_primitives::Key;

fn heap_from_slice<T>(slice: &[T]) -> BinaryHeap<T>
where
    T: Clone + PackedLayout + Ord,
{
    slice.iter().cloned().collect()
}

#[test]
fn new_binary_heap_works() {
    // `BinaryHeap::new`
    let mut heap = <BinaryHeap<i32>>::new();
    assert!(heap.is_empty());
    assert_eq!(heap.len(), 0);
    assert!(heap.iter().next().is_none());
    assert_eq!(heap.peek(), None);
    assert_eq!(heap.pop(), None);
    // `BinaryHeap::default`
    let mut default = <BinaryHeap<i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert!(default.iter().next().is_none());
    assert_eq!(default.peek(), None);
    assert_eq!(default.pop(), None);
    // `BinaryHeap::new` and `BinaryHeap::default` should be equal.
    assert_eq!(heap, default);
}

#[test]
fn from_iterator_works() {
    let some_primes = [1, 2, 3, 5, 7, 11, 13];
    assert_eq!(some_primes.iter().copied().collect::<BinaryHeap<_>>(), {
        let mut vec = BinaryHeap::new();
        for prime in &some_primes {
            vec.push(*prime)
        }
        vec
    });
}

#[test]
fn from_empty_iterator_works() {
    assert_eq!(
        [].iter().copied().collect::<BinaryHeap<i32>>(),
        BinaryHeap::new(),
    );
}

#[test]
fn pop_returns_greatest_element() {
    // push in order
    let mut heap = heap_from_slice(&[1, 2, 3]);

    assert_eq!(heap.len(), 3);
    assert_eq!(heap.pop(), Some(3));
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(1));
    assert!(heap.is_empty());

    // push out of order
    let mut heap = heap_from_slice(&[3, 2, 1]);

    assert_eq!(heap.len(), 3);
    assert_eq!(heap.pop(), Some(3));
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(1));
    assert!(heap.is_empty());
}

#[test]
fn peek_works() {
    let mut heap = <BinaryHeap<i32>>::new();
    heap.push(33);

    assert_eq!(heap.peek(), Some(&33));
}

// not sure we should have peek_mut, because it could violate the heap property?
// #[test]
// fn peek_mut_works() {
//     let mut heap = <BinaryHeap<i32>>::new();
//     heap.push(33);
//
//     let elem = heap.peek_mut().unwrap();
//     assert_eq!(heap.peek(), Some(&33));
// }

#[test]
fn spread_layout_push_pull_works() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let heap1 = heap_from_slice(&[b'a', b'b', b'c', b'd']);
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&heap1, &mut KeyPtr::from(root_key));
        // Load the pushed binary heap into another instance and check that
        // both instances are equal:
        let heap2 =
            <BinaryHeap<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(heap1, heap2);
        Ok(())
    })
}

#[test]
#[should_panic(expected = "encountered empty storage cell")]
fn spread_layout_clear_works() {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let heap1 = heap_from_slice(&[b'a', b'b', b'c', b'd']);
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&heap1, &mut KeyPtr::from(root_key));
        // It has already been asserted that a valid instance can be pulled
        // from contract storage after a push to the same storage region.
        //
        // Now clear the associated storage from `heap1` and check whether
        // loading another instance from this storage will panic since the
        // heap's length property cannot read a value:
        SpreadLayout::clear_spread(&heap1, &mut KeyPtr::from(root_key));
        let _ =
            <BinaryHeap<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        Ok(())
    })
    .unwrap()
}

#[test]
fn push_works_on_lazily_loaded_heap() {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let root_key = Key::from([0x42; 32]);
        let heap: BinaryHeap<u32> = heap_from_slice(&[1, 2, 3, 4]);
        SpreadLayout::push_spread(&heap, &mut KeyPtr::from(root_key));

        {
            let mut heap1 = std::mem::ManuallyDrop::new(
                <BinaryHeap<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(
                    root_key,
                )),
            );
            heap1.push(5);
        }

        // let mut heap1 =
        //     <BinaryHeap<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        // heap1.push(5);

        {
            let mut heap2 = <BinaryHeap<u8> as SpreadLayout>::pull_spread(
                &mut KeyPtr::from(root_key),
            );
            heap2.push(5);
        }
        Ok(())
    })
    .unwrap()
}

#[test]
fn clear_works_on_filled_heap() {
    let mut heap = heap_from_slice(&[b'a', b'b', b'c', b'd']);
    heap.clear();
    assert!(heap.is_empty());
}

#[test]
fn clear_works_on_empty_heap() {
    let mut heap = BinaryHeap::<u8>::default();
    heap.clear();
    assert!(heap.is_empty());
}