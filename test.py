"""
Example client for bigboi.

Run the server first (./bigboi) so the `bigboi.sock` file appears next to it,
then run this script from the same directory:  python3 test.py
"""

import socket
import hashlib


# ============================================================================
# COPY-PASTE THIS FUNCTION INTO YOUR OWN CODE
# ----------------------------------------------------------------------------
# `hashes` is an iterable of 16-byte MD5 digests (raw bytes, NOT hex strings).
# Returns a list of bools, one per input: True if the hash was newly inserted,
# False if it was already in the set.
#
# `socket_path` defaults to "bigboi.sock" in the current directory.
# ============================================================================

def check_md5s(hashes, socket_path="bigboi.sock"):
    hashes = list(hashes)
    count = len(hashes)

    # Open a fresh Unix-domain socket connection to the server.
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    connection.connect(socket_path)

    try:
        # 1) Send the batch size as a big-endian u64.
        connection.sendall(count.to_bytes(8, "big"))

        # 2) Send all the hashes back-to-back (count * 16 bytes total).
        connection.sendall(b"".join(hashes))

        # 3) Read exactly `count` response bytes; each is 1 (newly inserted)
        #    or 0 (already present).
        response = b""
        while len(response) < count:
            chunk = connection.recv(count - len(response))
            if not chunk:
                raise ConnectionError("server closed the connection early")
            response += chunk

        return [byte == 1 for byte in response]
    finally:
        connection.close()


# ============================================================================
# ============================================================================
# Everything below is just a demo of how to USE the function above.
# ============================================================================
# ============================================================================


def md5(text):
    """Helper: turn a string into its raw 16-byte MD5 digest."""
    return hashlib.md5(text.encode()).digest()


if __name__ == "__main__":
    # First batch: three distinct hashes. All should come back as "newly
    # inserted" (True), because the server's set starts empty.
    first_batch = [md5("apple"), md5("banana"), md5("cherry")]
    print("First batch:")
    print(" ", check_md5s(first_batch))
    # Expected: [True, True, True]

    # Second batch: "banana" again (already there), plus a new one.
    # Expected: [False, True]
    second_batch = [md5("banana"), md5("date")]
    print("Second batch:")
    print(" ", check_md5s(second_batch))

    # Third batch: all four hashes from before. All should be False now.
    # Expected: [False, False, False, False]
    third_batch = [md5("apple"), md5("banana"), md5("cherry"), md5("date")]
    print("Third batch:")
    print(" ", check_md5s(third_batch))
