mod my_atomic_queue;

use std::thread;
use std::sync::Arc;
use crate::my_atomic_queue::MyAtomicQueue;


fn main() {
    println!("Hello, atomic!");

    const ELEMENTS_COUNT: usize = 10;
        
    let queue = Arc::new(MyAtomicQueue::new(ELEMENTS_COUNT));

    let mut handles = vec![];

    for i in 0..ELEMENTS_COUNT {
        let temp_queue = queue.clone();

        let handle = thread::spawn(move || {
            println!("thread {} started", i);
            assert_eq!(temp_queue.push('a'), Ok(()));
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_queue_result = Arc::try_unwrap(queue);
    
    match final_queue_result {
        Ok(final_queue) => {
            assert_eq!(final_queue.len(), ELEMENTS_COUNT);
        }

        Err(final_queue) => {
            assert_eq!(final_queue.len(), ELEMENTS_COUNT);
        }
    }    
    
}


#[cfg(test)]
mod test;