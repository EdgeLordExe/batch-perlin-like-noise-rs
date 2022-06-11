# batch-perlin-like-noise-rs
Batched, discrete, parallelized perlin-like noise that uses a perlin-inspired algorithm to generate a large grid of boolean values.

DBPnoise works by first creating a series of "stamps" which are precalculated 2d grids that contain the dot product value between the vector from the cell to the centre of the stamp, and the vector
located at the centre of stamp. This series of stamps is then partially overlayed on top of each other and passed through the standard smoothstep function to generate the result.
