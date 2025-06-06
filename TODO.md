## TODO


- [ ] Add more builtins but one for each type
- [ ] Add tab completion for environment
- [ ] Add a "toggle_succinct" command
- [ ] Add "help" command
- [ ] Add `blst_sk_to_pk_in_g1/g2` and `blst_sign_pk_in_g1/g2`
- [ ] Add boolean type and equality operations
- [ ] Add `is_in_g1`, `is_in_g2` expressions
- [X] Add map_to_g1, map_to_g2 functions to turn 2 Coord1s/Coord2s into Point1/Point2
- [X] Encode Coord2 as pair. Parse it that way too.
- [X] Think about adding a Point6 type
- [X] Make `point1(0x4, sqrt(coord1(0x44)))` parseable
- [ ] Add scalar arithmetic (even though not supported by `blst`)
- [x] Add extended info for Point1 too
- [X] Add pretty printing library
- [ ] Validate size of all scalars/coord1/coord2s
- [ ] Add more scalar constants do you can do projections to torsion groups and G1/G2
- [x] Add embedding into F_p^12
- [x] Add Scalar multiplication for Point2
- [ ] Add polynomial representation in "info" command


-------------------

h2 mod p = 0x73eda753299d7d47d5034f7e659fd4ceb4cb5555b1500d525554aaa8aaaaaaad