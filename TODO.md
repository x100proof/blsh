## TODO

- [ ] Add map_to_g1, map_to_g2 functions to turn 2 Coord1s/Coord2s into Point1/Point2
- [X] Encode Coord2 as pair. Parse it that way too.
- [X] Think about adding a Point6 type
- [X] Make `point1(0x4, sqrt(coord1(0x44)))` parseable
- [ ] Add scalar arithmetic (even though not supported by `blst`)
- [ ] Add equality operations
- [x] Add extended info for Point1 too
- [ ] Add pretty printing library
- [ ] Validate size of all scalars/coord1/coord2s
- [ ] Add more scalar constants do you can do projections to torsion groups and G1/G2
- [x] Add embedding into F_p^12
- [x] Add Scalar multiplication for Point2
- [ ] Add polynomial representation in "info" command


This [page](https://hackmd.io/@benjaminion/bls12-381) renders the G2 points as pairs, so you're on right track.
```
p2 = ([0x02, 0x00],[0x013a59858b6809fca4d9a3b6539246a70051a3c88899964a42bc9a69cf9acdd9dd387cfa9086b894185b9a46a402be73,0x02d27e0ec3356299a346a09ad7dc4ef68a483c3aed53f9139d2f929a3eecebf72082e5e58c6da24ee32e03040c406d4f])
```
