# `blsh`

Pronounced "blush".

- A shell/REPL for doing BLS12-381 arithmetic.
- Backed by Supranational's [blst](https://github.com/supranational/blst) library.
- Inspired by [BLS12-381 For The Rest Of Us](https://hackmd.io/@benjaminion/bls12-381)[^1]

## Quick start

While I hope `blsh` can be used to do real BLS12-381 calculations it's reason for being is **for learning and building intuition**. Here are some examples:

### Bi-linearity

So, one of the first things you learn about BLS12-381 is it is bi-linear. A specific example of this general property is that the following equality should hold.

$$
e(sP, Q) = e(P, sQ)
$$

Let's try an example with $s = 0xcafebabe$, $P = g1$ and $Q = g2$ ( $g1$, $g2$ are generators for $G_1$ and $G_2$ the prime-order subgroups of $E_1$ and $E_2$ respectively).

```
blsh> embed(0xcafebabe|s * g1, g2)
[ { [ 0x0206cdae7064965190821b16dc1823995d376b4870a564c4b24bd68f1eed0b5014f833e7a56bed5040153a6ce29ad817
    , 0x0cd68b7001d5740857f5f872a0b0ac535cbc6184c53ab9f84b629241d96d0cff0911407293647791daeae1813bb84f71 ]
  , [ 0x0437f0770f2d20a924e5e454a97d5347fc4ae6c99245873e5fa88887f72d28d5e6019e80b0b2f7691b48aeb230523e4c
    , 0x09419f2f286b93543c8f144898141ecfae0d0aa9fdd878294830a174cd08b48f3e6c7800c06f1d689b00b705efb45260 ]
  , [ 0x085d8e1bfced24fd28e6a525b210ef4e8de438147b0f5fbba7c9b11fdcef344c22cdb05dfc76addf36e12fda7c307984
    , 0x0e7d83d1fb5bc0efea768eb90cf6c98c1f51509576e8796ed578cda3629312b7441ade29ad1cbed420ce8ab58443c7bf ] }
, { [ 0x02840794a8971ad433ea4cc2aff4fea3f5efa20c5319f3ec75ea2e25700a4a04c3017559ac3d9d3c290e0b272ff1f535
    , 0x065dcf73c47f6862a716f3628cd805bc156da37450b242516bcd1423314e0dddc1182e0b69d5217c4bd275056b9165e2 ]
  , [ 0x14af0ee1153a8cc5148295385cb5e19d1abbdb4d9b791c102821e6ff6a8cff967f60508d37248ba516ce37f88c82ba94
    , 0x110b7adeda2b2e8793795a295fe923f7d21dccd41f3f03d808ac2a79b7fd714c6e90f412cb6af1acd89931b47a26694d ]
  , [ 0x015ab5cd3607d2a399d8940127bf15e27ddd57e95c27dcfc5bc9350e3a6ef004a9caec50016996e43a021c90c33c6a58
    , 0x0ad74fe46627cf6fc76d2c44957c377d1d1af8ea6696b9ef42912e71e707dd5f66de4861d2af211ffa69127e60a35c65 ] } ]
 :Coord12
blsh> embed (g1, 0xcafebabe|s * g2)
[ { [ 0x0206cdae7064965190821b16dc1823995d376b4870a564c4b24bd68f1eed0b5014f833e7a56bed5040153a6ce29ad817
    , 0x0cd68b7001d5740857f5f872a0b0ac535cbc6184c53ab9f84b629241d96d0cff0911407293647791daeae1813bb84f71 ]
  , [ 0x0437f0770f2d20a924e5e454a97d5347fc4ae6c99245873e5fa88887f72d28d5e6019e80b0b2f7691b48aeb230523e4c
    , 0x09419f2f286b93543c8f144898141ecfae0d0aa9fdd878294830a174cd08b48f3e6c7800c06f1d689b00b705efb45260 ]
  , [ 0x085d8e1bfced24fd28e6a525b210ef4e8de438147b0f5fbba7c9b11fdcef344c22cdb05dfc76addf36e12fda7c307984
    , 0x0e7d83d1fb5bc0efea768eb90cf6c98c1f51509576e8796ed578cda3629312b7441ade29ad1cbed420ce8ab58443c7bf ] }
, { [ 0x02840794a8971ad433ea4cc2aff4fea3f5efa20c5319f3ec75ea2e25700a4a04c3017559ac3d9d3c290e0b272ff1f535
    , 0x065dcf73c47f6862a716f3628cd805bc156da37450b242516bcd1423314e0dddc1182e0b69d5217c4bd275056b9165e2 ]
  , [ 0x14af0ee1153a8cc5148295385cb5e19d1abbdb4d9b791c102821e6ff6a8cff967f60508d37248ba516ce37f88c82ba94
    , 0x110b7adeda2b2e8793795a295fe923f7d21dccd41f3f03d808ac2a79b7fd714c6e90f412cb6af1acd89931b47a26694d ]
  , [ 0x015ab5cd3607d2a399d8940127bf15e27ddd57e95c27dcfc5bc9350e3a6ef004a9caec50016996e43a021c90c33c6a58
    , 0x0ad74fe46627cf6fc76d2c44957c377d1d1af8ea6696b9ef42912e71e707dd5f66de4861d2af211ffa69127e60a35c65 ] } ]
 :Coord12```

Those look pretty equal to me!

### Points at infinity

In articles about BLS12-381 you'll often see it mentioned that the embedding for "points at infinity" (which act like the number $0$) is always $1$. You'll see something like this.

If $P = 0$ or $Q = 0$ (or both) then

$$
e(P,Q) = 1
$$

But what exactly is $1$? It's $1 \in \mathbb{F_p^{12}}$. But what the hell does that look like?

`blsh` to the rescue.

```
blsh> embed(p1inf, g2)
[ { [ 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001
    , 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ]
  , [ 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    , 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ]
  , [ 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    , 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] }
, { [ 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    , 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ]
  , [ 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    , 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ]
  , [ 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    , 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 ] } ]
 :Coord12
```



**FIXME**: Add more examples
- [ ] embed with point at infinity
- [ ] torsion points
- [ ] cofactor clearing

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

<table>
  <thead>
    <tr>
      <th>Suffix</th>
      <th>Bytes</th>
      <th>Description</th>
      <th>Type</th>
      <th>Examples</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>s</code></td>
      <td>32</td>
      <td>$\in \mathbb{F}_r$</td>
      <td>Scalar</td>
      <td> <code>`12:s`</code>, <code>0xdeadbeef:s</code> </td>
    </tr>
    <tr>
      <td></td>
      <td>48</td>
      <td>$\in \mathbb{F}_p$</td>
      <td><code>Coord1</code></td>
      <td> <code>`42`</code>, <code>0xcafebabe</code> </td>
    </tr>
    <tr>
      <td><code>p1</code></td>
      <td>48⋆</td>
      <td>compressed $\mathbb{F}_{p}$ pair</td>
      <td><code>Point1</code></td>
      <td><code>0x7abc...p1</code> (96 digits) </td>
    </tr>
    <tr>
      <td><code>u1</code></td>
      <td>48⋆</td>
      <td>uncompressed $\mathbb{F}_{p}$ pair</td>
      <td><code>Point1</code></td>
      <td><code>0x01fa...u1</code> (192 digits) </td>
    </tr>
    <tr>
      <td><code>p2</code></td>
      <td>96⋆</td>
      <td>compressed $\mathbb{F}_{p^2}$ pair</td>
      <td><code>Point2</code></td>
      <td><code>0x70da...p2</code> (192 digits) </td>
    </tr>
    <tr>
      <td><code>u2</code></td>
      <td>192⋆</td>
      <td>uncompressed $\mathbb{F}_{p^2}$ pair</td>
      <td><code>Point2</code></td>
      <td><code>0x002c...u2</code> (192 digits) </td>
    </tr>
  </tbody>
</table>

- You can optionally put a `:` or a `|` in front of the suffix to make things look cleaner e.g. `0xdeadbeef|s`
- ⋆ denotes that exactly 2× this many hex digits must be written in the literal
- See [ZCash serialization spec](https://github.com/zkcrypto/pairing/blob/34aa52b0f7bef705917252ea63e5a13fa01af551/src/bls12_381/README.md#serialization) for uncompressed/compressed form.

- **NOTE**: Compressed and uncompressed literals must be on the curve (but not necessarily in G1 or G2).
- **NOTE** This is not true for manually constructed points with `blsh`. You can create points that do not even lie on $E_1$ or $E_2$. This can be useful for security work.

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

```
(p1x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6b,  p1x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb)
```

```
((12, 0xf1), (0x12, 42))
```

### Coord<x> literals

We use square brackets to distinguish these pairs from their elliptic curve point brethren.

There are 4 types: `Coord1`, `Coord2`, `Coord6` and `Coord12`

`Coord12` corresponds to $\mathbb{F}_{p^{12}}$ coordinates which are:
- pairs of $\mathbb{F}_{p^{6}}$ coordinates, which are
- triples of $\mathbb{F}_{p^{2}}$ coordinates, which are
- pairs of $\mathbb{F}_p$ coordinates

#### Coord2 literals

Format                    |
--------------------------|
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

[^1]: Nice Seinfeld reference. "Festivus for the rest of us!"

