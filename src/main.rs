use crate::{identity_hasher::IdentityHasher, serve::serve};
use std::{
    collections::HashSet,
    hash::BuildHasherDefault,
    sync::{Arc, Mutex},
};
use tokio::net::UnixListener;

const CAPACITY: usize = 1_000_000;
const SOCKET_PATH: &str = "bigboi.sock";

pub type Set = HashSet<[u8; 16], BuildHasherDefault<IdentityHasher>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let set: Arc<Mutex<Set>> = Arc::new(Mutex::new(HashSet::with_capacity_and_hasher(
        CAPACITY,
        BuildHasherDefault::<IdentityHasher>::default(),
    )));

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

                tokio::spawn(async move {
                    if let Err(error) = serve(stream, set).await {
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

    Ok(())
}

mod identity_hasher;
mod serve;
