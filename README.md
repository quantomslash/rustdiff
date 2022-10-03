# RUSTDIFF

`RUSTDIFF` is a file diffing program very similar to `rdiff` that uses rolling hash algorithms to calculate `signature`, `delta` and `patch` the file using delta.

### File Signature

This program can calculate file signatures that can be used to test the integrity of the file.

`cargo run sign file.txt` (Replace file.txt with path to your file)

### Delta

Delta can be calculated by providing an original file and a modified file. This delta can be used to reproduce the changes to the modified file.

`cargo run delta file.txt file2.txt` 

Where `file.txt` is the source file,

And `file2.txt` is the modified file.

### Patch

Original file can be patched with delta to produce the modified file.

`cargo run patch file.txt delta.txt`

Where `file.txt` is the source file,

And `delta.txt` is the file containing `delta`.

## Custom options

You can specify your own chunk size and algorithm like so:

`cargo run sign file.txt 8 adler`

`cargo run delta file.txt file2.txt 8 adler`

`cargo run patch file.txt delta.txt 8 adler`

> NOTE --> The value of the chunk size and algorithm should be the same when creating delta and patching the file. Using different values for delta and patching will not work. The algorithm could either be 'adler' or 'fletcher'.

## Algorithms

There are two main algorithms that can be used to calculate hashes:

1. Fletcher32 - A well known has calculating algorithm,

2. Adler32 - Another well known algorithm, which is a modified version of Fletcher32 that uses prime modulus to calculate the hashes.

## Chunking Strategy

My first strategy involved generating delta with the same chunk size blocks as the signature file. These blocks were then matched with the signature blocks and matching indexes were added to the delta.

However, I modified it to scan the file byte by byte and if the data doesn't match, the bytes are added to the delta, instead of the whole blocks. This improved the efficiency and reduced the delta file sizes as byte by byte comparison was done.

## Why JSON?

**Just for readability!**

The serialisation format could be easily swapped with something that reduces the size even more. 

Example (CBOR) - `serde_cbor::to_writer(&f, &delta)?`

Or we could even use a custom format. The possibilities are endless.

As this is a demo project, I chose JSON as it makes it easy to quickly confirm the data. 

However I understand that it makes the delta sizes large and a binary format will be best fit for this case. Although keeping the chunk sizes large keeps the delta size low.  

## References

Following resources were consulted to learn and explore various hashing algorithms before writing the program.

https://en.wikipedia.org/wiki/Fletcher%27s_checksum#Fletcher-32

https://en.wikipedia.org/wiki/Adler-32

https://rsync.samba.org/tech_report/node2.html

https://stackoverflow.com/questions/6043708/how-reliable-is-the-adler32-checksum
