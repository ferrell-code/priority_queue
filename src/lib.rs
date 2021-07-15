use std::collections::HashMap;
use std::convert::TryInto;

pub trait PriorityQueue<Element> {
    /// create a new priority queue.
    fn new() -> Self;
    /// check whether the queue has no elements.
    fn is_empty(&self) -> bool;
    /// returns the highest-priority element but does not modify the queue.
    fn peek(&self) -> Option<Element>;
    /// add an element to the queue with an associated priority.
    fn insert(&mut self, element: Element, priority: u64);
    /// remove the element from the queue that has the highest priority, and return it.
    fn pop(&mut self) -> Option<Element>;
}

pub struct PriorityQueueImpl(HashMap<Vec<u8>, Vec<u8>>);

// Do not modify anything above ^^^

/// Priority Queue
///
/// Inserts encoded priority numbers as the key and encoded elements as the value into the hashmap backend.
/// In the case of elements with the same priority number we follow a FIFO (first in first out) data structure.
///
/// Make sure to only initialize with PriorityQueueImpl::new()
/// Manual initialization will cause undesireable behavior due to encoded values and will panic
impl PriorityQueue<Vec<u8>> for PriorityQueueImpl {
    /// Exact same semantics as `[HashMap::new()]`
    ///
    /// PriorityQueueImpl::new() creates a hash map with a capacity of 0
    /// so it will not allocate until it is inserted into
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// Exact same semantics as `[HashMap::is_empty()]`
    /// infaillible and will not panic
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets the value with the highest priority number
    /// Will return None in the case of an empty hashmap
    ///
    /// #[Panic]
    ///
    /// Will panic if elements were manually initialized and
    /// dont follow insert encoding convention
    fn peek(&self) -> Option<Vec<u8>> {
        let top_key = get_highest_priority(&self.0);
        let top_value = self.0.get(&top_key);
        match top_value {
            Some(element) => {
                let element_len: usize = element[0] as usize;
                Some(element[1..element_len + 1].to_vec())
            }
            None => None,
        }
    }

    /// Inserts priority and elements into hashmap using an encoding scheme
    ///
    /// The hash map key is an encoded priority of u64 into [u8; 8]
    ///
    /// the element is encoded by adding the length of the vec to the first value so vec![0, 0] encodes to vec![2, 0, 0]
    /// this allows us to support a FIFO data structure for when elements share priority numbers
    ///
    /// #[Panic]
    ///
    /// element must have a length of 255 or less or it will panic
    fn insert(&mut self, mut element: Vec<u8>, priority: u64) {
        // encodes keys for use in hash map
        let byte_key: Vec<u8> = priority.to_be_bytes().to_vec();

        // adds number of elements to front of vec
        let element_len: u8 = element
            .len()
            .try_into()
            .expect("element vec length was greater than 255");
        element.insert(0, element_len);

        // will insert key and value if key is not in use.
        // if key is in use the element is pushed onto the end of existing vec
        let current_value = self.0.get_mut(&byte_key);
        match current_value {
            None => {
                self.0.insert(byte_key, element);
            }
            Some(v) => {
                v.append(&mut element);
            }
        }
    }
    /// Pops highest priority element off queue
    /// FIFO for elements that have same priority
    ///
    /// #[Panic]
    ///
    /// will panic if data is inserted into hashmap manually and does not
    /// follow encoding convention of insert()
    fn pop(&mut self) -> Option<Vec<u8>> {
        let top_key = get_highest_priority(&self.0);
        let val = self.0.get_mut(&top_key);

        match val {
            Some(value) => {
                let element_len = value[0] as usize;
                let mut element: Vec<u8> = value.drain(..element_len + 1).collect::<Vec<u8>>();
                element.remove(0);
                if value.is_empty() {
                    // remove priority key and value if vec empty
                    self.0.remove(&top_key);
                }
                Some(element)
            }
            // empty hashmap returns none
            None => None,
        }
    }
}

/// iterates through all the keys to find the highest priority
/// if the hashmap is empty it returns vec![0; 8]
///
/// #[Panic]
///
/// Will panic if keys are not encoded as vec![u8; 8] according to insert() convention
/// Make sure to always use PriorityQueueImpl::new()
fn get_highest_priority(hmap: &HashMap<Vec<u8>, Vec<u8>>) -> Vec<u8> {
    let mut high_priority_num: u64 = 0;
    for key in hmap.keys() {
        // decodes keys from Vec<u8> to useable u64, all vecs should be 8 bytes long
        // due to the encoding of insert()
        let key_array: [u8; 8] = key
            .clone()
            .try_into()
            .expect("key was not encoded as [u8; 8]");
        let current_priority = u64::from_be_bytes(key_array);
        if current_priority > high_priority_num {
            high_priority_num = current_priority;
        }
    }
    // encodes the keys again to be used in the hashmap
    high_priority_num.to_be_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut queue = PriorityQueueImpl::new();
        assert!(queue.is_empty());

        queue.insert(vec![0], 5);
        assert!(!queue.is_empty());
        assert_eq!(queue.peek(), Some(vec![0]));

        queue.insert(vec![1], 10);
        queue.insert(vec![2], 3);
        queue.insert(vec![3], 4);
        queue.insert(vec![4], 6);

        assert_eq!(queue.pop(), Some(vec![1]));
        assert_eq!(queue.pop(), Some(vec![4]));
        assert_eq!(queue.pop(), Some(vec![0]));
        assert_eq!(queue.pop(), Some(vec![3]));
        assert_eq!(queue.pop(), Some(vec![2]));

        assert!(queue.is_empty());
        assert_eq!(queue.peek(), None);
    }

    #[test]
    fn large_number() {
        let mut queue = PriorityQueueImpl::new();
        queue.insert(vec![10], 10000);
        queue.insert(vec![5], 25000);
        queue.insert(vec![0], 0);
        assert_eq!(queue.peek(), Some(vec![5]));
    }

    #[test]
    fn key_is_zero() {
        let mut queue = PriorityQueueImpl::new();
        queue.insert(vec![10, 3], 0);
        assert_eq!(queue.peek(), Some(vec![10, 3]));
        assert_eq!(queue.pop(), Some(vec![10, 3]));
        assert_eq!(queue.pop(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn key_with_multiple_elements() {
        let mut queue = PriorityQueueImpl::new();
        queue.insert(vec![10; 5], 10);
        queue.insert(vec![5], 10);
        queue.insert(vec![100; 20], 10);
        assert_eq!(queue.pop(), Some(vec![10; 5]));
        assert_eq!(queue.pop(), Some(vec![5]));
        assert_eq!(queue.pop(), Some(vec![100; 20]));
        assert!(queue.is_empty());
    }

    #[test]
    fn element_empty_vec() {
        let mut queue = PriorityQueueImpl::new();
        queue.insert(Vec::new(), 10);
        queue.insert(Vec::new(), 10);
        assert_eq!(queue.peek(), Some(vec![]));
        assert_eq!(queue.pop(), Some(vec![]));
        assert_eq!(queue.peek(), Some(vec![]));
        assert_eq!(queue.pop(), Some(vec![]));
        assert_eq!(queue.pop(), None);
        assert!(queue.is_empty());
    }

    #[test]
    #[should_panic]
    fn element_too_long() {
        let mut queue = PriorityQueueImpl::new();
        queue.insert(vec![10; 1000], 10);
    }

    #[test]
    #[should_panic]
    fn key_not_8bytes() {
        let mut hmap = HashMap::new();
        hmap.insert(vec![5], vec![10]);
        hmap.insert(vec![1; 10], vec![5]);
        let mut queue = PriorityQueueImpl(hmap);
        assert_eq! {queue.peek(), None};
        assert_eq! {queue.pop(), None};
    }

    #[test]
    #[should_panic]
    fn manually_init_elements() {
        let mut hmap = HashMap::new();
        hmap.insert(vec![0; 8], vec![10, 0, 0]);
        let queue = PriorityQueueImpl(hmap);
        assert_eq! {queue.peek(), None};
    }
}
