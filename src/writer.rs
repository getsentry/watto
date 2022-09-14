use std::io::{Result, Write};

/// A wrapper around [`Write`] that keeps track of the bytes written.
///
/// The main usage is the [`Writer::align_to`] method which allows explicitly
/// aligning the output buffer by adding padding bytes.
#[derive(Debug)]
pub struct Writer<W: Write> {
    inner: W,
    pos: usize,
}

impl<W: Write> Writer<W> {
    /// Creates a new [`Writer`] wrapping a [`Write`] type.
    pub fn new(writer: W) -> Self {
        Self {
            inner: writer,
            pos: 0,
        }
    }

    /// Unwraps [`Writer`] into the inner [`Write`].
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Explicitly aligns the output buffer to `align` bytes by writing the
    /// necessary amount of padding bytes.
    pub fn align_to(&mut self, align: usize) -> Result<usize> {
        if !align.is_power_of_two() {
            panic!("aligned_to: align is not a power-of-two");
        }

        const PADDING_BYTES: &[u8] = &[0; 16];

        let len = self.pos % align;
        let len = if len == 0 { return Ok(0) } else { align - len };

        for _ in 0..(len / PADDING_BYTES.len()) {
            let _ = self.write(PADDING_BYTES)?;
        }

        self.write(&PADDING_BYTES[0..len % PADDING_BYTES.len()])
    }

    /// Explicitly aligns the output buffer to the alignment of `T` by writing the
    /// necessary amount of padding bytes.
    pub fn align_to_type<T>(&mut self) -> Result<usize> {
        self.align_to(core::mem::align_of::<T>())
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let written = self.inner.write(buf)?;
        self.pos += written;

        Ok(written)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}
