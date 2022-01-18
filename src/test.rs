use crate::my_atomic_queue::MyAtomicQueue;

#[test]
fn push_two_elements_succesful() {
    let q = MyAtomicQueue::new(2);

    assert_eq!(q.push('a'), Ok(()));
    assert_eq!(q.push('b'), Ok(()));
}


#[test]
fn pop_element() {
    let q = MyAtomicQueue::new(3);

    assert_eq!(q.push(10), Ok(()));
    assert_eq!(q.push(20), Ok(()));
    assert_eq!(q.push(30), Ok(()));
    
    assert_eq!(q.pop(), Some(10));
    assert!(!q.pop().is_none());
}

#[test]
fn get_capacity() {
    let q = MyAtomicQueue::<i32>::new(50);

    assert_eq!(q.capacity(), 50);
}

#[test]
fn get_len() {
    let q = MyAtomicQueue::new(100);
    
    assert_eq!(q.len(), 0);

    q.push(10).unwrap();
    assert_eq!(q.len(), 1);

    q.push(20).unwrap();
    assert_eq!(q.len(), 2);
}

#[test]
fn check_emptiness() {
    let q = MyAtomicQueue::new(100);
    
    assert!(q.is_empty());
    
    q.push(1).unwrap();
    assert!(!q.is_empty());
}

#[test]
fn check_fulness() {
    let q = MyAtomicQueue::new(1);

    assert!(!q.is_full());

    q.push(1).unwrap();
    assert!(q.is_full());
}

