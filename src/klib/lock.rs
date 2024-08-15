use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use crate::arch::lock::RawSpinLock;
// TODO poisoning and exceptiohns everywhere
// TODO manage writer starvation

/// Multiple reader, single writer lock
/// Spin lock for now TODO other lock types ?
pub struct RwLock<T: ?Sized> {
    /// Read lock
    read_lock: RawSpinLock,
    write_lock: RawSpinLock,
    // TODO I added that to make read() an immutable borrow, but is there a better way ?
    nreaders: UnsafeCell<usize>,
    data: UnsafeCell<T>,
}

// TODO not clear how that works
unsafe impl<T> Sync for RwLock<T> {}
// impl<T: ?Sized> for RwLock<T> {};

impl<T> RwLock<T> {
    /// Initialize a new RwLock wich data
    pub const fn new(data: T) -> Self {
        RwLock {
            read_lock: RawSpinLock::new(),
            write_lock: RawSpinLock::new(),
            nreaders: UnsafeCell::new(0),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Locks for read access
    /// It ensures that read access is possible before returning
    /// Otherwise it locks until the write access has been released
    pub fn read(&self) -> Result<RwLockReadGuard<T>, ()> {
        self.read_lock.lock();
        let readers = unsafe { &mut *self.nreaders.get() };
        *readers = *readers + 1;
        // if 1, this means that this is the first read lock
        if *readers == 1 {
            // reserve the write lock so that only reads are allowed until
            // nreaders decreases to 0
            self.write_lock.lock();
        }
        self.read_lock.release();
        // check if read lock is taken
        Ok(RwLockReadGuard { lock: self })
    }

    /// Locks for write access
    /// This function will lock until no one is reading anymore
    pub fn write(&self) -> Result<RwLockWriteGuard<T>, ()> {
        self.write_lock.lock();
        // check if write lock is taken
        Ok(RwLockWriteGuard { lock: self })
    }
}

/// Guard struct used for dropping the read lock
pub struct RwLockReadGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        let lock = &mut self.lock;
        lock.read_lock.lock();
        let readers = unsafe { &mut *lock.nreaders.get() };
        *readers = *readers - 1;
        if *readers == 0 {
            lock.write_lock.release();
        }
        lock.read_lock.release();
    }
}

/// Guard struct used for dropping the write lock
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.write_lock.release();
    }
}
