use super::error_name;
use crate::ffi::{
    ZSTD_DCtx, ZSTD_DStreamInSize, ZSTD_createDStream, ZSTD_decompressStream, ZSTD_freeDStream,
    ZSTD_inBuffer, ZSTD_isError, ZSTD_outBuffer,
};
use std::io::{Error, ErrorKind, Read};

/// An implementation of [`Read`] that decompress the data with zstd.
pub struct ZstdReader<F> {
    cx: *mut ZSTD_DCtx,
    buf: Vec<u8>,
    next: usize,
    from: F,
}

impl<F> ZstdReader<F> {
    pub fn new(from: F) -> Self {
        let block = unsafe { ZSTD_DStreamInSize() };

        Self {
            cx: unsafe { ZSTD_createDStream() },
            buf: vec![0; block],
            next: block,
            from,
        }
    }
}

impl<F> Drop for ZstdReader<F> {
    fn drop(&mut self) {
        assert_eq!(unsafe { ZSTD_isError(ZSTD_freeDStream(self.cx)) }, 0);
    }
}

impl<F: Read> Read for ZstdReader<F> {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let mut total = 0;
        let mut input = ZSTD_inBuffer {
            src: self.buf.as_ptr(),
            size: self.buf.len(),
            pos: self.next,
        };

        while !buf.is_empty() {
            // Setup input.
            if input.pos == input.size {
                let mut next = 0;

                while next < self.buf.len() {
                    let read = match self.from.read(&mut self.buf[next..]) {
                        Ok(v) => v,
                        Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                        Err(e) => return Err(e),
                    };

                    if read == 0 {
                        self.buf.truncate(next);
                        break;
                    }

                    next += read;
                }

                self.next = 0;

                input.src = self.buf.as_ptr();
                input.size = self.buf.len();
                input.pos = self.next;
            }

            // Setup output.
            let mut output = ZSTD_outBuffer {
                dst: buf.as_mut_ptr(),
                size: buf.len(),
                pos: 0,
            };

            // Decompress.
            let res = unsafe { ZSTD_decompressStream(self.cx, &mut output, &mut input) };

            if unsafe { ZSTD_isError(res) } != 0 {
                return Err(Error::new(ErrorKind::Other, error_name(res)));
            }

            // Update state.
            buf = &mut buf[output.pos..];
            total += output.pos;
            self.next = input.pos;

            // Check if completed.
            if self.buf.is_empty() && output.pos < output.size {
                break;
            }
        }

        Ok(total)
    }
}
