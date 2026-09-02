use super::metadata::BlogEntry;

// does &slice[index..], but as a const fn
pub const fn slice_from<T>(slice: &[T], index: usize) -> &[T] {
    assert!(index <= slice.len());

    unsafe {
        // SAFETY: index is in bounds of the slice or one-past-the end
        let ptr = slice.as_ptr().add(index);

        // SAFETY: ptr is valid, and slice.len() - index represents
        // the length of the slice after the index
        core::slice::from_raw_parts(ptr, slice.len() - index)
    }
}

pub const fn blog_entry_uid<N: BlogEntry>() -> &'static str {
    const {
        if N::UID == 0 {
            "0"
        } else {
            let buffer: &[u8] = &const {
                let mut buffer = [0; u32::MAX.ilog10() as usize + 1];

                let mut i = buffer.len();
                let mut n = N::UID;

                while 0 < i && n != 0 {
                    i -= 1;

                    buffer[i] = (n % 10) as u8 + b'0';
                    n /= 10;
                }

                buffer
            };

            let buffer_len = N::UID.ilog10() as usize + 1;

            let buffer = slice_from(buffer, buffer.len() - buffer_len);

            match core::str::from_utf8(buffer) {
                Ok(x) => x,
                Err(_) => unreachable!(),
            }
        }
    }
}
