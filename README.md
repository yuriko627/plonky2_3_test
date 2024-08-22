# Testing plonky2 and plonky3

This repository contains small testing code to compare `plonky2` and `plonky3`'s performance.
It contains circuits to prove the computation of
- 32nd Fibonacci number
- addition of 3500-degree polynomials redeced by mod `536870939` (this is an arbitrary number of degree/30-bits prime modulus I chose for my research interest)
by `plonky2` and `plonky3` for each.

You can run `cargo run --bin [filename]` to see the performance of prover time and verifier time.

### Performance comparison
#### 32nd Fibonacci
|             | prover time | verifier time |
| ----------- | ----------- |-------------- |
| plonky2 (64-bits prime field) | 1.28s | 85.87ms |
| plonky3 (31-bits prime field) | 2.57s | 75.3ms |

#### poly_add (30-bits prime modular arithmetic)
|             | prover time | verifier time |
| ----------- | ----------- |-------------- |
| plonky2 (64-bits prime field) | 7.28s | 237.39ms |
| plonky3 (31-bits prime field) | 937ms | 3.63s |

- measured on M1 Macbook Pro with 8 cores and 16GB memory
- poly_add by plonky2 doesn't do modular reduction yet
