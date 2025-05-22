# `blsh`

Pronounced "blush".

A shell/REPL for doing BLS12-381 arithmetic.

Backed by Supranational's [blst](https://github.com/supranational/blst) library.


## TODO

- [ ] Encode Coord2 as pair. Parse it that way too.
- [ ] Validate size of all scalars/coord1/coord2s
- [ ] Add more scalar constants do you can do projections to torsion groups and G1/G2
- [ ] Add embedding into F_p^12
- [ ] Add Scalar multiplication for Point2


This [page](https://hackmd.io/@benjaminion/bls12-381) renders the G2 points as pairs, so you're on right track.
```
p2 = ([0x02, 0x00],[0x013a59858b6809fca4d9a3b6539246a70051a3c88899964a42bc9a69cf9acdd9dd387cfa9086b894185b9a46a402be73,0x02d27e0ec3356299a346a09ad7dc4ef68a483c3aed53f9139d2f929a3eecebf72082e5e58c6da24ee32e03040c406d4f])
```