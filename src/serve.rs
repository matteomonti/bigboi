use crate::{Set, Writer};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub async fn serve(
    mut stream: UnixStream,
    set: Arc<Mutex<Set>>,
    writer: Writer,
) -> std::io::Result<()> {
    loop {
        let count = {
            let mut buffer = [0u8; 8];

            if stream.read_exact(&mut buffer).await.is_err() {
                return Ok(());
            }

            u64::from_be_bytes(buffer) as usize
        };

        let mut hashes = vec![[0u8; 16]; count];
        let mut insertions = vec![0u8; count];

        {
            let bytes = unsafe {
                // SAFETY: [u8; 16] is plain bytes; reinterpret the Vec as a flat byte slice.
                std::slice::from_raw_parts_mut(hashes.as_mut_ptr() as *mut u8, count * 16)
            };

            stream.read_exact(bytes).await?;
        }

        {
            let mut guard = set.lock().unwrap();

            for (hash, insertion) in hashes.iter().zip(insertions.iter_mut()) {
                if guard.insert(*hash) {
                    *insertion = 1;
                }
            }
        }

        {
            let mut guard = writer.lock().await;

            let Some(writer) = guard.as_mut() else {
                return Ok(());
            };

            for (hash, insertion) in hashes.iter().zip(insertions.iter()) {
                if *insertion == 1 {
                    writer.write_all(hash)?;
                }
            }
        }

        stream.write_all(&insertions).await?;
    }
}
