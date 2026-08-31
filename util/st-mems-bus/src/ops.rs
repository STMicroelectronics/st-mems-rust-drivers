use crate::CHUNK_SIZE;
use core::fmt::Debug;
use maybe_async::maybe_async;

#[maybe_async(AFIT)]
pub trait BusOperation {
    type Error: Debug;

    fn read_bytes(&mut self, rbuf: &mut [u8]) -> impl Future<Output = Result<(), Self::Error>>;
    fn write_bytes(&mut self, wbuf: &[u8]) -> impl Future<Output = Result<(), Self::Error>>;
    fn write_byte_read_bytes(
        &mut self,
        wbuf: &[u8; 1],
        rbuf: &mut [u8],
    ) -> impl Future<Output = Result<(), Self::Error>>;

    #[inline]
    fn read_from_register(
        &mut self,
        reg: u8,
        buf: &mut [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> {
        async move { self.write_byte_read_bytes(&[reg], buf).await }
    }

    #[inline]
    fn write_to_register(
        &mut self,
        reg: u8,
        buf: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>> {
        async move {
            let mut tmp: [u8; CHUNK_SIZE + 1] = [0; CHUNK_SIZE + 1];
            let mut reg = reg;
            for chunk in buf.chunks(CHUNK_SIZE) {
                tmp[0] = reg;
                tmp[1..1 + chunk.len()].copy_from_slice(chunk);
                self.write_bytes(&tmp[..1 + chunk.len()]).await?;

                reg = reg.wrapping_add(chunk.len() as u8);
            }
            Ok(())
        }
    }
}

#[maybe_async(AFIT)]
pub trait MemBankFunctions<M> {
    type Error;

    fn mem_bank_set(&mut self, val: M) -> impl Future<Output = Result<(), Self::Error>>;
    fn mem_bank_get(&mut self) -> impl Future<Output = Result<M, Self::Error>>;
}

#[maybe_async(AFIT)]
pub trait EmbAdvFunctions {
    type Error;

    fn ln_pg_write(
        &mut self,
        address: u16,
        buf: &[u8],
        len: u8,
    ) -> impl Future<Output = Result<(), Self::Error>>;

    fn ln_pg_read(
        &mut self,
        address: u16,
        buf: &mut [u8],
        len: u8,
    ) -> impl Future<Output = Result<(), Self::Error>>;
}

#[maybe_async(AFIT)]
pub trait RegisterOperation<D, E> {
    fn read(sensor: &mut D) -> impl Future<Output = Result<Self, E>>
    where
        Self: Sized;
    fn write(&self, sensor: &mut D) -> impl Future<Output = Result<(), E>>;
    fn read_more(sensor: &mut D, buff: &mut [u8]) -> impl Future<Output = Result<(), E>>;
}

#[maybe_async(AFIT)]
pub trait SensorOperation {
    type Error: Debug;

    fn read_from_register(
        &mut self,
        reg: u8,
        buf: &mut [u8],
    ) -> impl Future<Output = Result<(), Self::Error>>;

    fn write_to_register(
        &mut self,
        reg: u8,
        buf: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>>;
}
