We are going to implement a new feature, called "reseed", for reverse_rng.

There will be a new kind of line in the input files, which looks like: /reseed <offset>

This means that the following should happen:
 - We should generate a new 32-bit value the same as if this was a data point with the same offset
 - But then, instead of validating it, we should use this single 32-bit value to re-seed the rng
 - In the case of simd, this same operatrion should apply to each lane (vectorized).
 - Then, the offset is reset to zero. All subsequent data points AND reseeds are relative to this point!

 Start by gene
