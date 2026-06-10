use crate::{identity_hasher::IdentityHasher, serve::serve};
use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    hash::BuildHasherDefault,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};
use tokio::{net::UnixListener, sync::Mutex as AsyncMutex};

const SOCKET_PATH: &str = "bigboi.sock";
const PERSISTENCE_PATH: &str = "bigboi.bin";

pub type Set = HashSet<[u8; 16], BuildHasherDefault<IdentityHasher>>;
pub type Writer = Arc<AsyncMutex<Option<BufWriter<File>>>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let capacity: Option<usize> = std::env::args().nth(1).map(|arg| {
        arg.parse()
            .expect("Capacity must be a non-negative integer")
    });

    let mut set = {
        let hasher = BuildHasherDefault::<IdentityHasher>::default();

        match capacity {
            Some(capacity) => HashSet::with_capacity_and_hasher(capacity, hasher),
            None => HashSet::with_hasher(hasher),
        }
    };

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(PERSISTENCE_PATH)?;

    let mut file = {
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 16];
        let mut loaded: usize = 0;

        loop {
            match reader.read_exact(&mut buffer) {
                Ok(()) => {
                    set.insert(buffer);
                    loaded += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(error) => return Err(error),
            }
        }

        println!("Loaded {} entries from {}", loaded, PERSISTENCE_PATH);

        reader.into_inner()
    };

    file.seek(SeekFrom::End(0))?;

    let set = Arc::new(Mutex::new(set));
    let writer: Writer = Arc::new(AsyncMutex::new(Some(BufWriter::new(file))));

    let listener = {
        let _ = std::fs::remove_file(SOCKET_PATH);
        UnixListener::bind(SOCKET_PATH)?
    };

    println!("Listening on {}", SOCKET_PATH);

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let set = set.clone();
                let writer = writer.clone();

                tokio::spawn(async move {
                    if let Err(error) = serve(stream, set, writer).await {
                        eprintln!("Connection error: {}", error);
                    }
                });
            }

            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    println!("Shutting down");

    let taken = writer.lock().await.take();

    if let Some(mut buf_writer) = taken {
        buf_writer.flush()?;
        buf_writer.into_inner().unwrap().sync_all()?;
    }

    Ok(())
}

mod identity_hasher;
mod serve;
