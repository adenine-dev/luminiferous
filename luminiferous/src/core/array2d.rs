use std::ops::{Index, IndexMut};

use crate::prelude::*;

/// Static size, heap allocated, 2d array. The underlying data is stored row wise. Indexes `container[y][x]`.
///
/// # Examples
/// ```
/// # use luminiferous::{UExtent2, containers::Array2d};
/// let mut a = Array2d::from_1d(UExtent2::new(2, 2), vec![1, 2, 3, 4]);
///
/// assert_eq!(a[0][0], 1);
/// assert_eq!(a[0][1], 2);
/// assert_eq!(a[1][0], 3);
/// assert_eq!(a[1][1], 4);
/// ```
#[derive(Debug, Clone)]
pub struct Array2d<T> {
    extent: UExtent2,
    data: Vec<T>,
}

impl<T: Clone> Array2d<T> {
    /// Creates a new Array2d with the specified extent filled with the value `x`.
    ///
    /// # Examples
    /// ```
    /// # use luminiferous::{UExtent2, containers::Array2d};
    /// let a = Array2d::with_default(UExtent2::new(2, 2), 42);
    ///
    /// assert_eq!(a[0][0], 42);
    /// assert_eq!(a[1][0], 42);
    /// assert_eq!(a[0][1], 42);
    /// assert_eq!(a[1][1], 42);
    /// ```
    pub fn with_default(extent: UExtent2, x: T) -> Self {
        Self {
            extent,
            data: vec![x; (extent.x * extent.y) as usize],
        }
    }
}

impl<T> Array2d<T> {
    /// Returns a new Array2d with the specified extent and data. Data is truncated but not reallocated.
    ///
    /// # Panics
    /// Panics if data is not at least of length `extent.x * extent.y`.
    ///
    /// # Examples
    /// ```
    /// # use luminiferous::{UExtent2, containers::Array2d};
    ///
    /// let a = Array2d::from_1d(UExtent2::new(1, 2), vec![2, 3]);
    ///
    /// assert_eq!(a[0][0], 2);
    /// assert_eq!(a[0][1], 3);
    /// ```
    pub fn from_1d(extent: UExtent2, mut data: Vec<T>) -> Self {
        assert!(data.len() >= (extent.x * extent.y) as usize);

        data.truncate((extent.x * extent.y) as usize);
        Self { extent, data }
    }

    /// Returns the contained data as a 1d slice.
    pub fn as_1d(&self) -> &[T] {
        &self.data
    }

    /// Returns the contained data as a 1d mutable slice.
    pub fn as_1d_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns the extent of the contained data.
    ///
    /// # Examples
    /// ```
    /// # use luminiferous::{UExtent2, containers::Array2d};
    ///
    /// let a = Array2d::with_default(UExtent2::new(32, 42), 0);
    ///
    /// assert_eq!(a.get_extent(), UExtent2::new(32, 42));
    /// ```
    pub fn get_extent(&self) -> UExtent2 {
        self.extent
    }
}

impl<T> Index<usize> for Array2d<T> {
    type Output = [T];
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[(index * self.extent.x as usize)..((index + 1) * self.extent.x as usize)]
    }
}

impl<T> IndexMut<usize> for Array2d<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[(index * self.extent.x as usize)..((index + 1) * self.extent.x as usize)]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn as_1d() {
        let mut a = Array2d::with_default(UExtent2::new(3, 4), 42);

        a.as_1d_mut().iter_mut().for_each(|x| *x += 1);
        assert_eq!(a[1][1], 43);
        assert_eq!(a[2][0], 43);
        assert_eq!(a[1][2], 43);
        assert_eq!(a[2][2], 43);

        a[1][0] = 32;
        assert_eq!(a.as_1d()[4], 32);
    }
}
