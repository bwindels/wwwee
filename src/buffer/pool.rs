use std::cell::RefCell;
use std::slice;
use std::vec::Vec;
use super::buffer::Buffer;

#[derive(Debug, PartialEq)]
enum BorrowError {
  Depleted,
  BeyondLargest(usize)
}

#[derive(Debug, PartialEq)]
enum CategoryError {
  Unaligned(usize),
  Unsorted
}

fn align_ptr(offset: *mut u8, alignment: usize) -> *mut u8 {
  let remainder = offset as usize % alignment;
  if remainder == 0 {
    offset
  }
  else {
    unsafe { offset.offset((alignment - remainder) as isize) }
  }
}

struct Category {
  pub amount: usize,
  pub size: usize
}

impl Category {
  fn total_size(&self) -> usize {
    self.amount * self.size
  }
}

struct BufferPool<'a> {
  backing_store: Vec<u8>,
  slots: Vec<RefCell<&'a mut [u8]>>,
  categories: [Category; 2]
}

impl<'a> BufferPool<'a> {
  pub fn new(alignment: usize, small_category: Category, large_category: Category) -> Result<BufferPool<'a>, CategoryError> {
    let categories = [small_category, large_category];
    
    if let Some(err) = Self::check_categories(&categories, alignment) {
      return Err(err);
    }

    let total_size = categories.iter().fold(0, |t, cat| t + cat.total_size());
    let total_amount = categories.iter().fold(0, |t, cat| t + cat.amount);
    let mut backing_store = Vec::with_capacity(total_size + alignment);
    let mut slots = Vec::with_capacity(total_amount);
    let start_ptr = backing_store.as_mut_ptr();
    let aligned_start_ptr = align_ptr(start_ptr, alignment);

    Self::fill_slots(aligned_start_ptr, &mut slots, &categories);

    Ok(BufferPool {
      backing_store,
      slots,
      categories
    })
  }

  pub fn borrow_buffer(&'a self, size_hint: usize) -> Result<Buffer<'a>, BorrowError> {
    let cat_index = self.categories.iter().position(|cat| cat.size >= size_hint);
    if let Some(cat_index) = cat_index {
      let categories_before = &self.categories[0 .. cat_index];
      let amount_before = categories_before.iter().fold(0, |t, cat| t + cat.amount);
      let big_enough_slots = &self.slots[amount_before..];
      let slice_ref = big_enough_slots.iter().filter_map(|slot| {
        slot.try_borrow_mut().ok()
      }).nth(0); 
      slice_ref.map(|s| Buffer::from_slice(s)).ok_or(BorrowError::Depleted)
    }
    else {
      let largest_size = self.categories.iter().last().unwrap().size;
      Err(BorrowError::BeyondLargest(largest_size))
    }
  }

  fn fill_slots(aligned_start_ptr: *mut u8,
                slots: &mut Vec<RefCell<&'a mut [u8]>>,
                categories: &[Category])
  {
    let buffer_sizes = categories.iter().flat_map(|cat| {
      (0..cat.amount).map(move |_| cat.size)
    });
    buffer_sizes.fold(aligned_start_ptr, |start_ptr, size| {
      let slice = unsafe { slice::from_raw_parts_mut(start_ptr, size) };
      slots.push(RefCell::new(slice));
      return unsafe { start_ptr.offset(size as isize) };
    });
  }

  fn check_categories(categories: &[Category], alignment: usize) -> Option<CategoryError> {
    if let Some(unaligned_idx) = categories.iter().
        position(|cat| (cat.size % alignment) != 0)
    {
      return Some(CategoryError::Unaligned(unaligned_idx));
    }

    let is_unsorted = categories.windows(2).find(|cats| {
      cats[1].size < cats[0].size
    });
    if is_unsorted.is_some() {
      return Some(CategoryError::Unsorted);
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::{BufferPool, Category, BorrowError, CategoryError};

  const ALIGNMENT: usize = 40;
  const SMALL_SIZE: usize = 40;
  const LARGE_SIZE: usize = 80;

  fn test_pool<'a>() -> BufferPool<'a> {
    BufferPool::new(
      ALIGNMENT,
      Category {size: SMALL_SIZE, amount: 2},
      Category {size: LARGE_SIZE, amount: 2}
    ).unwrap()
  }

  #[test]
  fn test_alloc_small_buffer() {
    let pool = test_pool();
    let small = pool.borrow_buffer(SMALL_SIZE - 1).unwrap();
    assert_eq!(small.remaining(), SMALL_SIZE);
  }

  #[test]
  fn test_alloc_large_buffer() {
    let pool = test_pool();
    let large = pool.borrow_buffer(SMALL_SIZE + 1).unwrap();
    assert_eq!(large.remaining(), LARGE_SIZE);
  }

  #[test]
  fn test_alignment() {
    let pool = test_pool();
    let small = pool.borrow_buffer(SMALL_SIZE - 1).unwrap();
    let ptr = small.as_slice().as_ptr();
    assert_eq!(ptr as usize % ALIGNMENT, 0);
  }

  #[test]
  fn test_overfit_when_fit_unavailable() {
    let pool = test_pool();
    let small1 = {pool.borrow_buffer(SMALL_SIZE - 1).unwrap()};
    let small2 = {pool.borrow_buffer(SMALL_SIZE - 1).unwrap()};
    let small3 = {pool.borrow_buffer(SMALL_SIZE - 1).unwrap()};
    assert_eq!(small1.remaining(), SMALL_SIZE);
    assert_eq!(small2.remaining(), SMALL_SIZE);
    assert_eq!(small3.remaining(), LARGE_SIZE);
  }

  #[test]
  fn test_full() {
    let pool = test_pool();
    let buffer1 = {pool.borrow_buffer(SMALL_SIZE)};
    let buffer2 = {pool.borrow_buffer(SMALL_SIZE)};
    let buffer3 = {pool.borrow_buffer(SMALL_SIZE)};
    let buffer4 = {pool.borrow_buffer(SMALL_SIZE)};
    let buffer5 = {pool.borrow_buffer(SMALL_SIZE)};
    assert!(buffer1.is_ok());
    assert!(buffer2.is_ok());
    assert!(buffer3.is_ok());
    assert!(buffer4.is_ok());
    assert_eq!(buffer5.err(), Some(BorrowError::Depleted));
  }

  #[test]
  fn test_size_not_available() {
    let pool = test_pool();
    let large1 = {pool.borrow_buffer(LARGE_SIZE)};
    let large2 = {pool.borrow_buffer(LARGE_SIZE)};
    let large3 = {pool.borrow_buffer(LARGE_SIZE)};
    assert!(large1.is_ok());
    assert!(large2.is_ok());
    assert_eq!(large3.err(), Some(BorrowError::Depleted));
  }

  #[test]
  fn test_unaligned_sizes() {
    let pool = BufferPool::new(
      40,
      Category {size: 40, amount: 1},
      Category {size: 70, amount: 1}
    );
    assert_eq!(pool.err(), Some(CategoryError::Unaligned(1)));
  }

  #[test]
  fn test_unsorted_categories() {
    let pool = BufferPool::new(
      40,
      Category {size: 80, amount: 1},
      Category {size: 40, amount: 1}
    );
    assert_eq!(pool.err(), Some(CategoryError::Unsorted));
  }
}
