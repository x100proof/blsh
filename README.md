# `blsh`

Pronounced "blush".

A shell/REPL for doing BLS12-381 arithmetic.

Backed by Supranational's [blst](https://github.com/supranational/blst) library.

## Terminology

- field modulus $p$. A large prime constructed from the following equation
  $${1 \over 3}(x - 1)^2(x^4 - x^2 + 1) + x \text{\ \ \ \ \ \ where\ } x = \text{-0xd201000000010000}$$
- $E_1$ and $E_2$ are the two elliptic curves.
  * $E_1 = E(\mathbb{F}_p)$
  * $E_2 = E(\mathbb{F}_{p^2})$
- $r$ is the large prime factor of the _order_ of E1
  $$(x^4 - x^2 + 1) \text{\ \ \ \ \  where\ } x = \text{-0xd201000000010000}$$
- $r = \text{0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001}$

## Types

This language has the following types



<table>
  <thead>
    <tr>
      <th><code>blsh</code> type</th>
      <th>Description</th>
      <th>In <code>blst</code></th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>Scalar</code></td>
      <td>$\in \mathbb{F}_r$</td>
      <td><code>blst_scalar</code></td>
    </tr>
    <tr>
      <td><code>Coord1</code></td>
      <td>$\in \mathbb{F}_p$</td>
      <td><code>blst_fp</code></td>
    </tr>
    <tr>
      <td><code>Coord2</code></td>
      <td>$(\mathbb{F}_p,\ \mathbb{F}_p) \in \mathbb{F}_{p^2}$</td>
      <td><code>blst_fp2</code></td>
    </tr>
    <tr>
      <td><code>Coord6</code></td>
      <td>$(\mathbb{F}_{p^2},\ \mathbb{F}_{p^2},\ \mathbb{F}_{p^2}) \in \mathbb{F}_{p^6}$</td>
      <td><code>blst_fp6</code></td>
    </tr>
    <tr>
      <td><code>Coord12</code></td>
      <td>$(\mathbb{F}_{p^6},\ \mathbb{F}_{p^6}) \in \mathbb{F}_{p^{12}}$</td>
      <td><code>blst_fp12</code></td>
    </tr>
    <tr>
      <td><code>Point1</code></td>
      <td>$(\mathbb{F}_p,\ \mathbb{F}_p) \in E_1$</td>
      <td><code>blst_p1</code></td>
    </tr>
    <tr>
      <td><code>Point2</code></td>
      <td>$(\mathbb{F}_{p^2},\ \mathbb{F}_{p^2}) \in E_2$ point in elliptic curve $E_2$</td>
      <td><code>blst_scalar</code></td>
    </tr>
  </tbody>
</table>

Just like `blst` does, we distinguish between `Coord2` and `Point1` even though they are both pairs of $\mathbb{F_{p}}$. They are semantically distinct.

## Literals

In order to make reading (and parsing!) expressions easy we use a novel literal representation inspired by Rust

### Number literals

| Suffix          | Bytes  | Description           | Type         | Examples
------------------|--------|-----------------------|--------------|-----------------------
   `s`           | 32     |  $\in \mathbb{F}_r$   | Scalar       | `12:s`, `0xdeadbeef:s`
                 | 48     |  $\in \mathbb{F}_p$   | Coord1       | `42`, `0xcafebabe`
   `p1`          | 48⋆    | compressed $\mathbb{F}_{p}$ pair     | Point1 | `0x7abc...p1` (96  digits)
   `u1`          | 96⋆    | uncompressed $\mathbb{F}_{p}$ pair   | Point1 | `0x01fa...u1` (192 digits)
   `p2`          | 96⋆    | compressed $\mathbb{F}_{p^2}$ pair   | Point2 | `0x70da...p2` (192 digits)
   `u2`          | 192⋆   | uncompressed $\mathbb{F}_{p^2}$ pair | Point2 | `0x002c...u2` (384 digits)

- You can optionally put a `:` or a `|` in front of the suffix to make things look cleaner e.g. `0xdeadbeef|s`
- ⋆ denotes that exactly 2× this many hex digits must be written in the literal
- See [ZCash serialization spec](https://github.com/zkcrypto/pairing/blob/34aa52b0f7bef705917252ea63e5a13fa01af551/src/bls12_381/README.md#serialization) for uncompressed/compressed form.

- **NOTE**: Compressed and uncompressed literals must be on the curve (but not necessarily in G1 or G2).
- **NOTE** This is not true for manually constructed points with `blsh`. This can be useful for security work.

### Point literals

#### Point1 literals

  Format               |
-----------------------|
 `(`Coord1`,`Coord1`)` |

Examples:
- `(0xff, 0cabd)`
- `(1,2)`
- `(12, 0xf1)`

#### Point2 literals

 Format                     |
 ---------------------------|
  `(` Coord2 `,` Coord2 `)` |

Examples:

```( p1x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6b ,p1x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb )
```

- `((12, 0xf1), (0x12, 42))

### Coord<x> literals

We use square brackets to distinguish these pairs from their elliptic curve point brethren.

There are 4 types: `Coord1`, `Coord2`, `Coord6` and `Coord12`

`Coord12` corresponds to $\mathbb{F}_{p^{12}}$ coordinates which are:
- pairs of $\mathbb{F}_{p^{6}}$ coordinates, which are
- triples of $\mathbb{F}_{p^{2}}$ coordinates, which are
- pairs of $\mathbb{F}_p$ coordinates

#### Coord2 literals

Format |
`[` Coord1 `,` Coord1 `]` |


#### Coord6 literals

**FIXME**: Talk about the grammar

We provide two literal formats but they will always be printed in nested format.

Format    |           |
----------|-----------|
Flattened |  `[`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`]` |
Nested    | `[[`Coord1`,`Coord1`],[`Coord1`,`Coord1`],`Coord1`,`Coord1`]]`



#### Coord12 literals

**FIXME**: Talk about the grammar

We provide two literal formats but they will always be printed in nested format.

Format    |           |
----------|-----------|
Flattened |  `[`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`,`Coord1`]` |
Nested    | `[[[`Coord1`,`Coord1`],[`Coord1`,`Coord1`],`Coord1`,`Coord1`]],[[`Coord1`,`Coord1`],[`Coord1`,`Coord1`],`Coord1`,`Coord1`]]]`






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


## GitHub, why isn't this rendering correctly?

- $(\mathbb{F}_p, \mathbb{F}_p) \in \mathbb{F}_{p^2}$ does not render correctly, but
- $(\mathbb{F}_p, \mathbb{F}_p)$ does, and so does
- $\mathbb{F}_{p^2}$

Even "breaking it all up on one line" doesn't render properly:
$(\mathbb{F}_p, \mathbb{F}_p)$
$\in$
$\mathbb{F}_{p^2}$
