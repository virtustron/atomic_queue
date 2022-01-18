
use std::marker::PhantomData;
use std::cell::UnsafeCell;

use core::mem::MaybeUninit;
use core::sync::atomic::{self, AtomicUsize, Ordering};

use ::std::boxed::Box;

use crossbeam_utils::CachePadded;
use crossbeam_utils::Backoff;


struct MySlot<T> {
    stamp: AtomicUsize,                     // position in queue to compair with "head" and "tail"
    value: UnsafeCell<MaybeUninit<T>>,      // value of element in queue
}


pub struct MyAtomicQueue<T> {
    head: CachePadded<AtomicUsize>,     // `CachePadded` - to avoid cache locality.
    tail: CachePadded<AtomicUsize>,     
    buffer: *mut MySlot<T>,             // holding slots
    capacity: usize,                    // capacity of queue
    _marker: PhantomData<T>,            // indicates that this scruct "contains" elements of type `T` (lifetime corresponding purpuses)
}

unsafe impl<T: Send> Sync for MyAtomicQueue<T> {}
unsafe impl<T: Send> Send for MyAtomicQueue<T> {}

impl<T> MyAtomicQueue<T> {

    pub fn new(capacity: usize) -> MyAtomicQueue<T> {
        assert!(capacity > 0, "capacity must be non-zero");

        let head = 0;
        let tail = 0;

        // Allocate a buffer of `capacity` slots initialized with stamps.
        let buffer = {
            let boxed: Box<[MySlot<T>]> = (0..capacity)
                .map(|i| {
                    MySlot {
                        stamp: AtomicUsize::new(i),
                        value: UnsafeCell::new(MaybeUninit::uninit()),
                    }
                })
                .collect();
            Box::into_raw(boxed) as *mut MySlot<T>
        };

        MyAtomicQueue {
            head: CachePadded::new(AtomicUsize::new(head)),
            tail: CachePadded::new(AtomicUsize::new(tail)),
            buffer,
            capacity,
            _marker: PhantomData,
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        let backoff = Backoff::new(); 
        let mut tail = self.tail.load(Ordering::Relaxed);

        loop {
            let slot = unsafe { &*self.buffer.add(tail) };         // shift pointer to `tail`
            let stamp = slot.stamp.load(Ordering::Acquire);        // "after" can't move "before"

            // If the tail and the stamp match, we may attempt to push.
            if tail == stamp {
                let new_tail = if tail + 1 < self.capacity {
                    tail + 1
                } else {
                    return Err(value);
                };

                // Try moving the tail.
                match self.tail.compare_exchange_weak(
                    tail,
                    new_tail,
                    Ordering::SeqCst,    // success
                    Ordering::Relaxed,   // failure
                ) {
                    Ok(_) => {
                        // Write the value into the slot and update the stamp.
                        unsafe {
                            slot.value.get().write(MaybeUninit::new(value));
                        }
                        slot.stamp.store(tail + 1, Ordering::Release);   // "before" can't move after
                        return Ok(());
                    }
                    Err(t) => {
                        tail = t;
                        backoff.spin();
                    }
                }
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.snooze();
                tail = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        let backoff = Backoff::new();
        let mut head = self.head.load(Ordering::Relaxed);

        loop {
            let slot = unsafe { &*self.buffer.add(head) };
            let stamp = slot.stamp.load(Ordering::Acquire);    // "after" can't move "before"

            // ??? If the the stamp is ahead of the head by 1, we may attempt to pop.
            if head + 1 == stamp {
                let new = if head + 1 < self.capacity {
                    head + 1
                } else {
                    return None;
                };

                // Try moving the head.
                match self.head.compare_exchange_weak(
                    head,
                    new,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Read the value from the slot and update the stamp.
                        let msg = unsafe { slot.value.get().read().assume_init() };
                        slot.stamp.store(head, Ordering::Release);
                        return Some(msg);
                    }
                    Err(h) => {
                        head = h;
                        backoff.spin();
                    }
                }
            } else if stamp == head {
                atomic::fence(Ordering::SeqCst);                    // prevents the compiler and CPU from reordering 
                                                                    //   certain types of memory operations around it
                let tail = self.tail.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if tail == head {
                    return None;
                }

                backoff.spin();
                head = self.head.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.snooze();
                head = self.head.load(Ordering::Relaxed);
            }
        }
    }

    // Returns the capacity of the queue.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);

        tail == head
    }

    // Returns `true` if the queue is full.
    pub fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::SeqCst);
        let head = self.head.load(Ordering::SeqCst);
       
        tail - head + 1 == self.capacity
    }

    // Returns the number of elements in the queue.
    pub fn len(&self) -> usize {
        loop {
            let tail = self.tail.load(Ordering::SeqCst);
            let head = self.head.load(Ordering::SeqCst);

            if tail == head {
                return 0
            } else {
                return tail - head + 1
            }
        }
    }
}