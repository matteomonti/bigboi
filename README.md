# bigboi

An append-only, in-RAM HashSet of MD5 hashes, served over a Unix-domain socket.
The set is kept resident in memory (sized for multi-TB hosts) and mirrored to
`./bigboi.bin` so that, after a graceful shutdown, the
on-disk file perfectly reflects what was in RAM.

Because MD5 digests are already uniformly distributed, the set uses an identity
hasher (the first 8 bytes of each hash are taken as the bucket key) — no
re-hashing is performed.

## Build

```
cargo build --release
```

The binary lands at `target/release/bigboi`.

## Run

```
./bigboi [capacity]
```

`capacity` is an optional positional argument — the initial capacity of the
in-memory `HashSet`. When omitted, the set starts at the default size and grows
on demand (`HashSet::new`-style). When given, the set is pre-sized with
`HashSet::with_capacity`, which on multi-TB deployments is the difference
between a smooth start and hours of incremental rehashing as the set fills.
There is no upper limit other than available RAM; pick something close to the
number of hashes you expect to hold.

On boot, `bigboi` looks for `./bigboi.bin` and replays it into the set, then
opens a Unix socket at `./bigboi.sock`. Press Ctrl+C for a graceful shutdown:
in-flight batches that were already persisted finish being acknowledged,
anything mid-flight is dropped, and the file is flushed and `fsync`ed before
exit.

## Persistence

The on-disk file is `./bigboi.bin`, resolved relative to the working directory
the server was launched from (same goes for `./bigboi.sock`). It is a flat
concatenation of 16-byte MD5 digests, no header and no framing — append
`./bigboi.bin` files together and you get the union of their sets.

Newly-inserted hashes are written into a `BufWriter` *before* the server
acknowledges the client. The buffer itself only reaches disk on graceful
shutdown, when `bigboi` flushes it and calls `fsync` before exiting.

What this gets you:

- **After a graceful shutdown** (Ctrl+C), `./bigboi.bin` is exactly the set
  that was in RAM. Restart and you pick up where you left off.
- **No false acknowledgements**: on shutdown the writer is detached under a
  mutex before being flushed, so any batch that did not make it into the
  buffer is dropped without a response rather than acknowledged. A client
  that sees a `1` for a hash can trust the server has it on disk after the
  next clean shutdown.
- **Hard kills (`kill -9`, power loss) will lose recently-acknowledged
  hashes** still sitting in the BufWriter. Don't `kill -9` if you care.

## Protocol

Each connection is a stream of request/response batches:

1. Client sends batch size as a big-endian `u64`.
2. Client sends that many 16-byte MD5 digests back-to-back.
3. Server inserts them all, persists the newly-inserted ones, then sends back
   one byte per input: `1` if newly inserted, `0` if already present.

There is no header on the response — the client already knows how many bytes
to expect.

## Python example

[`test.py`](./test.py) shows how to talk to the server. The top of the file
is a self-contained `check_md5s()` function you can copy-paste into your own
code; below it is a small demo that exercises three batches against a running
server.

```
./bigboi &
python3 test.py
```
