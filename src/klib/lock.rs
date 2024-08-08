use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::arch::lock::SpinLock;

// TODO poisoning and exceptiohns everywhere

/// Multiple reader, single writer lock
/// Spin lock for now TODO other lock types ?
pub struct RwLock<'rwlock, T> {
    /// Read lock
    read_lock: SpinLock,
    write_lock: SpinLock,
    nreaders: usize,
    data: T,
    _marker: PhantomData<&'rwlock T>
}

impl<'rwlock, T> RwLock<'rwlock, T> {
    /// Initialize a new RwLock wich data
    pub fn new(data: T) -> Self {
        RwLock {
            read_lock: SpinLock::new(), 
            write_lock: SpinLock::new(), 
            nreaders: 0,
            data,
            _marker: PhantomData
        }
    }

    /// Locks for read access
    /// It ensures that read access is possible before returning
    /// Otherwise it locks until the write access has been released
    pub fn read(&'rwlock mut self) -> Result<RwLockReadGuard<'rwlock, T>, ()> {
        self.read_lock.lock();
        self.nreaders += 1;
        // if 1, this means that this is the first read lock
        if self.nreaders == 1 {
            // reserve the write lock so that only reads are allowed until
            // nreaders decreases to 0
            self.write_lock.lock();
        }
        self.read_lock.release();
        // check if read lock is taken
        Ok(RwLockReadGuard {
            lock: self
        })
    }

    /// Locks for write access
    /// This function will lock until no one is reading anymore
    pub fn write(&'rwlock mut self) -> Result<RwLockWriteGuard<T>, ()> {
        self.write_lock.lock();
        // check if write lock is taken
        Ok(RwLockWriteGuard {
            lock: self
        })
    }
}

/// Guard struct used for dropping the read lock
pub struct RwLockReadGuard<'a, T> {
    lock: &'a mut RwLock<'a, T>
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<T> Drop for  RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        let lock = &mut self.lock;
        lock.read_lock.lock();
        lock.nreaders -= 1;
        if lock.nreaders == 0 {
            lock.write_lock.release();
        }
        lock.read_lock.release();
    }
}

/// Guard struct used for dropping the write lock
pub struct RwLockWriteGuard<'a, T> {
    lock: &'a mut RwLock<'a, T>
}

impl<T> Deref for RwLockWriteGuard <'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<T> DerefMut for RwLockWriteGuard <'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lock.data
    }
}

impl<T> Drop for  RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
       self.lock.write_lock.release(); 
    }
}

