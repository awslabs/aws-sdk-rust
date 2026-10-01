/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Intrusive FIFO bookkeeping shared by admission indexes.
//!
//! [`IntrusiveOrder`] owns only the order endpoints and length. Domain records
//! own their [`IntrusiveLinks`], which keeps scheduling residence and list
//! membership in one representation. Before removing a record, the domain
//! owner repairs its neighboring records and passes the removed links to
//! [`IntrusiveOrder::remove`] so the order can update its endpoints.
//!
//! Mutations derive checked replacement state before changing endpoints. The
//! caller remains responsible for invoking its domain invariant check after a
//! completed transition.

#[cfg(any(debug_assertions, test))]
use std::fmt;
use std::num::NonZeroUsize;

/// Links owned by a record while it occupies an admission order.
#[derive(Clone, Copy, Debug)]
pub(super) struct IntrusiveLinks<K> {
    /// Previous record in FIFO order.
    pub(super) previous: Option<K>,
    /// Next record in FIFO order.
    pub(super) next: Option<K>,
}

/// Endpoints and length of one admission-owned intrusive FIFO view.
#[derive(Debug, Default)]
pub(super) enum IntrusiveOrder<K> {
    /// No record is linked.
    #[default]
    Empty,
    /// At least one record is linked from `head` through `tail`.
    Active {
        /// Oldest linked record.
        head: K,
        /// Youngest linked record.
        tail: K,
        /// Number of linked records.
        len: NonZeroUsize,
    },
}

impl<K> IntrusiveOrder<K>
where
    K: Copy + Eq,
{
    /// Returns the oldest linked record.
    pub(super) fn head(&self) -> Option<K> {
        match self {
            Self::Empty => None,
            Self::Active { head, .. } => Some(*head),
        }
    }

    /// Returns the youngest linked record.
    pub(super) fn tail(&self) -> Option<K> {
        match self {
            Self::Empty => None,
            Self::Active { tail, .. } => Some(*tail),
        }
    }

    /// Returns the number of linked records.
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Active { len, .. } => len.get(),
        }
    }

    /// Appends `key` and returns the links its record must store.
    pub(super) fn push_back(&mut self, key: K) -> IntrusiveLinks<K> {
        let previous = self.tail();
        match self {
            order @ Self::Empty => {
                *order = Self::Active {
                    head: key,
                    tail: key,
                    len: NonZeroUsize::MIN,
                };
            }
            Self::Active { tail, len, .. } => {
                let next_len = len
                    .checked_add(1)
                    .expect("admission order length exhausted");
                *tail = key;
                *len = next_len;
            }
        }
        IntrusiveLinks {
            previous,
            next: None,
        }
    }

    /// Removes `key` after its owner has repaired neighboring record links.
    pub(super) fn remove(&mut self, key: K, links: IntrusiveLinks<K>) {
        let replacement = match self {
            Self::Empty => unreachable!("removed a record from an empty admission order"),
            Self::Active { head, tail, len } => {
                let head = *head;
                let tail = *tail;
                let len = *len;
                debug_assert_eq!(head == key, links.previous.is_none());
                debug_assert_eq!(tail == key, links.next.is_none());
                if len == NonZeroUsize::MIN {
                    Self::Empty
                } else {
                    Self::Active {
                        head: if head == key {
                            links.next.expect("removed order head had no successor")
                        } else {
                            head
                        },
                        tail: if tail == key {
                            links
                                .previous
                                .expect("removed order tail had no predecessor")
                        } else {
                            tail
                        },
                        len: NonZeroUsize::new(
                            len.get()
                                .checked_sub(1)
                                .expect("admission order length underflowed"),
                        )
                        .expect("nonempty admission order lost its length"),
                    }
                }
            }
        };
        *self = replacement;
    }

    /// Checks endpoints, length, and record-owned links for one FIFO view.
    #[cfg(any(debug_assertions, test))]
    pub(super) fn assert_consistent(
        &self,
        expected: usize,
        record_bound: usize,
        label: &str,
        mut links_for: impl FnMut(K) -> IntrusiveLinks<K>,
    ) where
        K: fmt::Debug,
    {
        let mut current = self.head();
        let mut previous = None;
        let mut traversed = 0;
        while let Some(key) = current {
            assert!(traversed < record_bound, "{label} contains a cycle");
            let links = links_for(key);
            assert_eq!(
                links.previous, previous,
                "{label} contains inconsistent backward links"
            );
            previous = Some(key);
            current = links.next;
            traversed += 1;
        }
        assert_eq!(expected, traversed, "{label} omitted linked records");
        assert_eq!(self.len(), traversed, "{label} length was incorrect");
        assert_eq!(self.tail(), previous, "{label} tail was not reachable");
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[derive(Default)]
    struct LinkedOrder {
        order: IntrusiveOrder<u8>,
        links: HashMap<u8, IntrusiveLinks<u8>>,
    }

    impl LinkedOrder {
        fn push_back(&mut self, key: u8) {
            let links = self.order.push_back(key);
            if let Some(previous) = links.previous {
                self.links
                    .get_mut(&previous)
                    .expect("previous record disappeared")
                    .next = Some(key);
            }
            assert!(self.links.insert(key, links).is_none());
            self.assert_consistent();
        }

        fn remove(&mut self, key: u8) {
            let links = self.links.remove(&key).expect("record disappeared");
            if let Some(previous) = links.previous {
                self.links
                    .get_mut(&previous)
                    .expect("previous record disappeared")
                    .next = links.next;
            }
            if let Some(next) = links.next {
                self.links
                    .get_mut(&next)
                    .expect("next record disappeared")
                    .previous = links.previous;
            }
            self.order.remove(key, links);
            self.assert_consistent();
        }

        fn keys(&self) -> Vec<u8> {
            let mut keys = Vec::new();
            let mut current = self.order.head();
            while let Some(key) = current {
                assert!(keys.len() < self.links.len());
                keys.push(key);
                current = self.links.get(&key).expect("record disappeared").next;
            }
            keys
        }

        fn assert_consistent(&self) {
            self.order
                .assert_consistent(self.links.len(), self.links.len(), "test order", |key| {
                    *self.links.get(&key).expect("record disappeared")
                });
        }
    }

    #[test]
    fn insertion_and_removal_preserve_fifo_order() {
        let mut linked = LinkedOrder::default();
        linked.assert_consistent();

        for key in 1..=4 {
            linked.push_back(key);
        }
        assert_eq!(vec![1, 2, 3, 4], linked.keys());

        linked.remove(2);
        assert_eq!(vec![1, 3, 4], linked.keys());
        linked.remove(1);
        assert_eq!(vec![3, 4], linked.keys());
        linked.remove(4);
        assert_eq!(vec![3], linked.keys());
        linked.remove(3);
        assert!(linked.keys().is_empty());
    }

    #[test]
    #[should_panic(expected = "test order contains inconsistent backward links")]
    fn consistency_check_rejects_broken_backward_links() {
        let mut linked = LinkedOrder::default();
        linked.push_back(1);
        linked.push_back(2);
        linked.links.get_mut(&2).unwrap().previous = None;
        linked.assert_consistent();
    }

    #[test]
    #[should_panic(expected = "test order contains a cycle")]
    fn consistency_check_rejects_cycles() {
        let mut linked = LinkedOrder::default();
        linked.push_back(1);
        linked.push_back(2);
        linked.push_back(3);
        linked.links.get_mut(&3).unwrap().next = Some(2);
        linked.assert_consistent();
    }

    #[test]
    fn length_overflow_leaves_order_unchanged() {
        let mut order = IntrusiveOrder::Active {
            head: 1,
            tail: 1,
            len: NonZeroUsize::new(usize::MAX).unwrap(),
        };

        let result = catch_unwind(AssertUnwindSafe(|| order.push_back(2)));

        assert!(result.is_err());
        assert_eq!(Some(1), order.head());
        assert_eq!(Some(1), order.tail());
        assert_eq!(usize::MAX, order.len());
    }

    #[test]
    fn failed_removal_leaves_order_unchanged() {
        let mut order = IntrusiveOrder::Active {
            head: 1,
            tail: 2,
            len: NonZeroUsize::new(2).unwrap(),
        };

        let result = catch_unwind(AssertUnwindSafe(|| {
            order.remove(
                1,
                IntrusiveLinks {
                    previous: None,
                    next: None,
                },
            )
        }));

        assert!(result.is_err());
        assert_eq!(Some(1), order.head());
        assert_eq!(Some(2), order.tail());
        assert_eq!(2, order.len());
    }
}
