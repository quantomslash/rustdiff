# RUSTDIFF

Rust diff is file diffing program very similar to `rdiff` that uses rolling hash algorithms to calculate signature, delta and patch the file using delta.

### File Signature

This program can calculate file signature that can be used to test the integrity of the file.

### Delta

Delta can be calculated by providing an original file and a modified file. This delta can be used to reproduce the changes to the modified file.

### Patch

Original file can be patched with delta to produce the modified file.


## Algorithms

There are two main algorithms that can be used to calculate hashes:

1. Fletcher32 - A well known has calculating algorithm,
2. Adler32 - Another well known algorithm, which is modified version of Fletcher32 that uses prime modulas to calculate the hashes.








## References

https://en.wikipedia.org/wiki/Fletcher%27s_checksum#Fletcher-32
https://en.wikipedia.org/wiki/Adler-32







One implementation involved