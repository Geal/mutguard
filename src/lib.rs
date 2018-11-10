use std::ops::{Deref,DerefMut,Drop};

pub struct MutGuard<T> {
  inner: T,
}

impl<T> Deref for MutGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

pub trait Guard {
  fn finish(&mut self);
}

impl<T: Guard> MutGuard<T> {
  pub fn new(inner: T) -> MutGuard<T> {
    MutGuard { inner }
  }

  pub fn guard<'a>(&'a mut self) -> MutGuardBorrow<'a, T> {
    MutGuardBorrow { inner: self }
  }

  pub fn into_inner(self) -> T {
    self.inner
  }
}

impl<T> MutGuard<T> {
  pub fn wrap<'a, F>(inner: T, f: F) -> MutGuard<MutGuardWrapper<'a, T>>
    where F: 'a + for<'r> FnMut(&'r mut T) {
    let wrapper = MutGuardWrapper {
      inner, f: Box::new(f)
    };
    MutGuard::new(wrapper)
  }
}

pub struct MutGuardBorrow<'a, T: Guard> {
  inner: &'a mut MutGuard<T>,
}

impl<'a, T: Guard> Deref for MutGuardBorrow<'a,T> {
  type Target = T;

  fn deref(&self) -> &T {
      &self.inner.inner
  }
}

impl<'a,T: Guard> DerefMut for MutGuardBorrow<'a,T> {
  fn deref_mut(&mut self) -> &mut T {
      &mut self.inner.inner
  }
}

impl<'a,T: Guard> Drop for MutGuardBorrow<'a,T> {
  fn drop(&mut self) {
    self.inner.inner.finish();
  }
}

pub struct MutGuardWrapper<'a, T> {
  inner: T,
  f: Box<'a+FnMut(&mut T)>,
}

impl<'a, T: 'a> MutGuardWrapper<'a, T> {
  pub fn new<F>(inner: T, f: F) -> MutGuardWrapper<'a, T>
    where F: 'a + for<'r> FnMut(&'r mut T) {
    MutGuardWrapper { inner, f: Box::new(f) }
  }
}

impl<'a, T> Guard for MutGuardWrapper<'a, T> {
  fn finish(&mut self) {
    (self.f)(&mut self.inner);
  }
}

impl<'a, T> Deref for MutGuardWrapper<'a,T> {
  type Target = T;

  fn deref(&self) -> &T {
      &self.inner
  }
}

impl<'a,T> DerefMut for MutGuardWrapper<'a,T> {
  fn deref_mut(&mut self) -> &mut T {
      &mut self.inner
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug)]
  struct Bank {
    accounts: Vec<i32>,
  }

  impl Bank {
    pub fn new(accounts: Vec<i32>) -> Bank {
      Bank { accounts }
    }

    pub fn transfer(&mut self, account1: usize, account2: usize, amount: i32) {
      self.accounts[account1] -= amount;
      self.accounts[account2] += amount;
    }
  }

  impl Guard for Bank {
    fn finish(&mut self) {
      assert!(self.accounts.iter().any(|v| *v < 0), "accounts should not become negative");
    }
  }

  #[test]
  #[should_panic(expected = "accounts should not become negative")]
  fn invariant_bank() {
    let mut ibank = MutGuard::new(
      Bank::new(vec!(10, 0, 20, 50))
    );

    println!("bank: {:?}", *ibank);

    {
    ibank.guard().transfer(0, 1, 5);
    println!("bank: {:?}", *ibank);

    ibank.guard().transfer(2, 3, 30);
    println!("bank: {:?}", *ibank);
    }
  }

  #[test]
  fn bank() {
    let mut bank = Bank::new(vec!(10, 0, 20, 50));
    println!("bank: {:?}", bank);

    bank.transfer(0, 1, 5);
    println!("bank: {:?}", bank);

    bank.transfer(2, 3, 30);
    // now accounts are [5, 5, -10, 80]
    println!("bank: {:?}", bank);
  }

  #[test]
  fn count_access() {
    let mut counter = 0;
    let v = Vec::new();

    {
      let mut iv = MutGuard::wrap(v, |_| counter += 1);

      iv.guard().push(1);
      iv.guard().push(2);
      iv.guard().push(3);
    }

    assert_eq!(counter, 3);
  }
}
