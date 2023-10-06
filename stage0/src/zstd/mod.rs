use crate::ffi::{
    ZSTD_CStreamInSize, ZSTD_CStreamOutSize, ZSTD_EndDirective, ZSTD_compressStream2,
    ZSTD_createCStream, ZSTD_freeCStream, ZSTD_getErrorName, ZSTD_inBuffer, ZSTD_isError,
    ZSTD_outBuffer, ZstdContex,
};
use std::cmp::min;
use std::ffi::CStr;
use std::io::{Error, ErrorKind, Write};
use std::ptr::null;

/// An implementation of [`Write`] that compress the data with zstd before writing to the underlying
/// [`Write`].
pub struct ZstdWriter<D> {
    cx: *mut ZstdContex,
    buf: Vec<u8>,
    block: usize,
    dest: D,
}

impl<D> ZstdWriter<D> {
    pub fn new(dest: D) -> Self {
        Self {
            cx: unsafe { ZSTD_createCStream() },
            buf: vec![0; unsafe { ZSTD_CStreamOutSize() }],
            block: unsafe { ZSTD_CStreamInSize() },
            dest,
        }
    }

    fn error_name(code: usize) -> &'static str {
        unsafe { CStr::from_ptr(ZSTD_getErrorName(code)).to_str().unwrap() }
    }
}

impl<D> Drop for ZstdWriter<D> {
    fn drop(&mut self) {
        assert_eq!(unsafe { ZSTD_isError(ZSTD_freeCStream(self.cx)) }, 0);
    }
}

impl<D: Write> Write for ZstdWriter<D> {
    fn write(&mut self, mut buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;

        while !buf.is_empty() {
            // Setup input.
            let mut input = ZSTD_inBuffer {
                src: buf.as_ptr(),
                size: min(buf.len(), self.block),
                pos: 0,
            };

            // Setup output.
            let mut output = ZSTD_outBuffer {
                dst: self.buf.as_mut_ptr(),
                size: self.buf.len(),
                pos: 0,
            };

            // Compress.
            let remain = unsafe {
                ZSTD_compressStream2(
                    self.cx,
                    &mut output,
                    &mut input,
                    ZSTD_EndDirective::ZSTD_e_continue,
                )
            };

            if unsafe { ZSTD_isError(remain) } != 0 {
                return Err(Error::new(ErrorKind::Other, Self::error_name(remain)));
            }

            // Write the destination.
            self.dest.write_all(&self.buf[..output.pos])?;

            // Move to next data.
            written += input.pos;
            buf = &buf[input.pos..];
        }

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        loop {
            // Setup input.
            let mut input = ZSTD_inBuffer {
                src: null(),
                size: 0,
                pos: 0,
            };

            // Setup output.
            let mut output = ZSTD_outBuffer {
                dst: self.buf.as_mut_ptr(),
                size: self.buf.len(),
                pos: 0,
            };

            // Flush.
            let remain = unsafe {
                ZSTD_compressStream2(
                    self.cx,
                    &mut output,
                    &mut input,
                    ZSTD_EndDirective::ZSTD_e_end,
                )
            };

            if unsafe { ZSTD_isError(remain) } != 0 {
                break Err(Error::new(ErrorKind::Other, Self::error_name(remain)));
            }

            // Write the destination.
            self.dest.write_all(&self.buf[..output.pos])?;

            if remain == 0 {
                break Ok(());
            }
        }
    }
}
