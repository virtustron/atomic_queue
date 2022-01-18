mod my_atomic_queue;

use crate::my_atomic_queue::MyAtomicQueue;

fn main() {
    println!("Hello, atomic!");

    let q = MyAtomicQueue::new(100);
    
    assert_eq!(q.len(), 0);

    q.push(10).unwrap();
    assert_eq!(q.len(), 1);

    q.push(20).unwrap();
    assert_eq!(q.len(), 2);
    
}


#[cfg(test)]
mod test;