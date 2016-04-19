// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


//! The frame module provides a structred way to send and receive
//! message through streams.
//!
//! ## Data Framing
//!
//! ```ignore
//! 0                   1                   2                   3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! | Frame Start   |  Payload Len                  |  Payload  |
//! +-----------------------------------------------------------+
//! |           Payload Data Continued          |   Frame End   |
//! +-----------------------------------------------------------+
//!
//! Start Frame:    8 bits, must be 0x01
//! Payload Len:    16 bits
//! Payload Data:   (Payload Len) bytes
//! End Frame:      8 bits, must be 0x17
//! ```


use std::mem;


pub const START_BYTE:    u8 = 0x01;
pub const END_BYTE:      u8 = 0x17;


pub fn new(buf: &[u8]) -> Vec<u8> {
    let mut ret_buf = Vec::<u8>::with_capacity(buf.len() + 4);
    let buf_len = buf.len() as u16;

    trace!("new buf.len: {}", buf.len());
    trace!("payload_len: {}", buf_len);

    ret_buf.push(START_BYTE);
    ret_buf.push((buf_len >> 8) as u8);
    ret_buf.push(buf_len as u8);
    ret_buf.extend_from_slice(buf);
    ret_buf.push(END_BYTE);

    ret_buf
}

pub fn from_raw_parts(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    if buf.len() < 5 {
        trace!("buf.len not large enough to process");
        return None;
    }

    if buf[0] != START_BYTE {
        trace!("buf[0] was not START_BYTE");
        let mut new_buf = Vec::<u8>::with_capacity(1024);
        mem::swap(&mut new_buf, buf);
        return None;
    }

    let mask = 0xFFFFu16;
    let mut payload_len = ((buf[1] as u16) << 8) & mask;
    payload_len |= buf[2] as u16;

    trace!("buf.len: {}", buf.len());
    trace!("Payload Len: {}", payload_len);

    let payload_len = payload_len as usize;
    if (buf.len() - 4) < payload_len {
        trace!("Not enough in buf for expected payload\nExpected: {}\nActual: {}",
               payload_len,
               (buf.len() - 4));
        return None;
    }

    if buf[payload_len + 3] != END_BYTE {
        trace!("END_BYTE was not at expected location. Swapping for a fresh buffer");
        let mut new_buf = Vec::<u8>::with_capacity(1024);
        mem::swap(&mut new_buf, buf);
        return None;
    }

    let mut ret_buf = Vec::<u8>::with_capacity(payload_len);
    ret_buf.extend_from_slice(&buf[3..payload_len]);

    let buf_len = buf.len();
    let mut remaining_buf = Vec::<u8>::with_capacity(buf.len() - (payload_len + 4));
    remaining_buf.extend_from_slice(&buf[(payload_len + 3)..buf_len]);
    mem::swap(buf, &mut remaining_buf);

    trace!("Complete frame read");

    Some(ret_buf)
}
