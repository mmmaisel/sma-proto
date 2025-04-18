/******************************************************************************\
    sma-proto - A SMA Speedwire protocol library
    Copyright (C) 2024-2025 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
use core::{ops::Deref, result::Result};

/// Interface to a variable length storage container.
pub trait SmaContainer<T>: Deref<Target = [T]> + Default {
    /// Adds an element to the collection.
    fn push(&mut self, value: T) -> Result<(), T>;
}

#[cfg(feature = "std")]
impl<T> SmaContainer<T> for Vec<T> {
    fn push(&mut self, value: T) -> Result<(), T> {
        self.push(value);
        Ok(())
    }
}

#[cfg(feature = "heapless")]
impl<T, const N: usize> SmaContainer<T> for heapless::Vec<T, N> {
    fn push(&mut self, value: T) -> Result<(), T> {
        self.push(value)
    }
}
