/******************************************************************************\
    sma-proto - A SMA Speedwire protocol library
    Copyright (C) 2024 Max Maisel

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

use super::{Error, Result};
use byteorder::ByteOrder;
#[cfg(not(feature = "std"))]
use core::{fmt::Debug, prelude::rust_2021::derive, result::Result::Ok};

/// A std::io::Cursor like buffer interface with byteorder support and no_std
/// compatibility.
#[derive(Debug)]
pub struct Cursor<T> {
    buffer: T,
    pos: usize,
}

impl<T: AsRef<[u8]>> Cursor<T> {
    /// Constructs a new cursor object on top of a slice.
    pub fn new(buffer: T) -> Self {
        Self { buffer, pos: 0 }
    }

    #[allow(clippy::len_without_is_empty)]
    /// Returns the length of the underlying buffer.
    pub fn len(&self) -> usize {
        self.buffer.as_ref().len()
    }

    /// Returns the remaining length in bytes of the underlying buffer.
    pub fn remaining(&self) -> usize {
        self.buffer.as_ref().len() - self.pos
    }

    /// Checks if the underlying buffer has the expected amount of space left.
    pub fn check_remaining(&self, expected: usize) -> Result<()> {
        if self.remaining() < expected {
            return Err(Error::BufferTooSmall {
                size: self.len(),
                expected: self.pos + expected,
            });
        }

        Ok(())
    }

    /// Returns the cursor position in the underlying buffer.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Sets the cursor position in the underlying buffer.
    pub fn set_position(&mut self, position: usize) {
        self.pos = position
    }

    /// Advances it cursor position by the given amount of bytes.
    pub fn skip(&mut self, count: usize) {
        self.pos += count;
    }

    /// Reads data from the underlying buffer to the given slice and advances
    /// cursor position.
    /// Panics if there is not enough data remaining to fill the slice.
    pub fn read_bytes(&mut self, dst: &mut [u8]) {
        dst.copy_from_slice(
            &self.buffer.as_ref()[self.pos..(self.pos + dst.len())],
        );
        self.pos += dst.len();
    }

    /// Reads a 8bit integer value from the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough data remaining.
    pub fn read_u8(&mut self) -> u8 {
        let val = self.buffer.as_ref()[self.pos];
        self.pos += 1;
        val
    }

    /// Reads a 16bit integer value from the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough data remaining.
    pub fn read_u16<B: ByteOrder>(&mut self) -> u16 {
        let val = B::read_u16(&self.buffer.as_ref()[self.pos..]);
        self.pos += 2;
        val
    }

    /// Reads a 24bit integer value from the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough data remaining.
    pub fn read_u24<B: ByteOrder>(&mut self) -> u32 {
        let val = B::read_u24(&self.buffer.as_ref()[self.pos..]);
        self.pos += 3;
        val
    }

    /// Reads a 32bit integer value from the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough data remaining.
    pub fn read_u32<B: ByteOrder>(&mut self) -> u32 {
        let val = B::read_u32(&self.buffer.as_ref()[self.pos..]);
        self.pos += 4;
        val
    }

    /// Reads a 64bit integer value from the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough data remaining.
    pub fn read_u64<B: ByteOrder>(&mut self) -> u64 {
        let val = B::read_u64(&self.buffer.as_ref()[self.pos..]);
        self.pos += 8;
        val
    }

    /// Reads a 16bit integer value from the underlying buffer at a given
    /// offset from the cursor position without advancing the cursor position.
    /// Panics if there is not enough data remaining.
    pub fn peek_u16<B: ByteOrder>(&self, offset: usize) -> u16 {
        B::read_u16(&self.buffer.as_ref()[(self.pos + offset)..])
    }

    /// Reads a 24bit integer value from the underlying buffer at a given
    /// offset from the cursor position without advancing the cursor position.
    /// Panics if there is not enough data remaining.
    pub fn peek_u24<B: ByteOrder>(&self, offset: usize) -> u32 {
        B::read_u24(&self.buffer.as_ref()[(self.pos + offset)..])
    }

    /// Reads a 32bit integer value from the underlying buffer at a given
    /// offset from the cursor position without advancing the cursor position.
    /// Panics if there is not enough data remaining.
    pub fn peek_u32<B: ByteOrder>(&self, offset: usize) -> u32 {
        B::read_u32(&self.buffer.as_ref()[(self.pos + offset)..])
    }
}

impl Cursor<&mut [u8]> {
    /// Writes the given slice to the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough space remaining to write the slice.
    pub fn write_bytes(&mut self, src: &[u8]) {
        self.buffer[self.pos..(self.pos + src.len())].copy_from_slice(src);
        self.pos += src.len();
    }

    /// Writes a 8bit integer value to the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough space remaining.
    pub fn write_u8(&mut self, val: u8) {
        self.buffer[self.pos] = val;
        self.pos += 1;
    }

    /// Writes a 16bit integer value to the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough space remaining.
    pub fn write_u16<B: ByteOrder>(&mut self, val: u16) {
        B::write_u16(&mut self.buffer[self.pos..], val);
        self.pos += 2;
    }

    /// Writes a 24bit integer value to the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough space remaining.
    pub fn write_u24<B: ByteOrder>(&mut self, val: u32) {
        B::write_u24(&mut self.buffer[self.pos..], val);
        self.pos += 3;
    }

    /// Writes a 32bit integer value to the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough space remaining.
    pub fn write_u32<B: ByteOrder>(&mut self, val: u32) {
        B::write_u32(&mut self.buffer[self.pos..], val);
        self.pos += 4;
    }

    /// Writes a 64bit integer value to the underlying buffer and advances
    /// cursor position.
    /// Panics if there is not enough space remaining.
    pub fn write_u64<B: ByteOrder>(&mut self, val: u64) {
        B::write_u64(&mut self.buffer[self.pos..], val);
        self.pos += 8;
    }
}
