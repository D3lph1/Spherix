use spherix_math::vector::Vector3;
use spherix_util::math::is_power_of_two;
use std::hash::Hasher;

/// Basic trait for random number generators. Provides fundamental random number generation
/// capabilities.
pub trait Rng {
    /// Generates a random 64-bit unsigned integer.
    fn next_u64(&mut self) -> u64;

    /// Generates a random 32-bit unsigned integer in the range [0, max_value).
    fn next_u32(&mut self, max_value: u32) -> u32;

    /// Generates a random double-precision floating-point number.
    fn next_f64(&mut self) -> f64;

    /// Generates a random single-precision floating-point number.
    fn next_f32(&mut self) -> f32;

    fn next_bool(&mut self) -> bool;

    fn next_u32_inclusive(&mut self, min: u32, max: u32) -> u32 {
        self.next_u32(max - min + 1) + min
    }

    /// Skips `n` random numbers.  Useful for advancing the RNG's internal state without
    /// consuming the generated values.  This is often used for seeding or skipping
    /// ahead in a sequence.
    fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.next_u64();
        }
    }
}

/// Trait for RNGs that can be forked into position-dependent sub-generators.
/// This enables creating multiple independent but related RNGs, each tied to a specific
/// location.
pub trait RngForkable: Rng {
    /// The type of positional RNG that can be created by this RNG.  This specifies the
    /// type of positional RNG this generator can produce.
    type Pos: RngPos;

    /// Creates a new positional RNG from this RNG.  This creates a new `RngPos` instance,
    /// seeded from the current state of the `RngForkable` instance.  The new `RngPos` will
    /// likely have its own internal state, independent but related to the original.
    fn fork_pos(&mut self) -> Self::Pos;
}

/// Trait for random number generators that can generate values dependent on positional data.
/// This allows for spatially varying randomness.
pub trait RngPos {
    type Item: RngForkable;

    /// Creates a new `RngPos` instance from a given random number generator.
    /// This initializes the positional RNG.
    fn from_rng<R: Rng>(rng: &mut R) -> Self;

    /// Returns a value based on the given position.  This is the core functionality –
    /// generating a value at a specific location.
    fn at(&self, pos: Vector3) -> Self::Item;

    /// Returns a value based on a given string.  This allows for deterministic generation
    /// based on a hash of the input string. Useful for repeatable results.
    fn by_hash(&self, s: String) -> Self::Item;
}

/// Minecraft-compatible implementation of XoroShiro128+ algorithm. It differs from
/// "xoroshiro" crate's one.
pub struct XoroShiro {
    lo: u64,
    hi: u64,
}

impl XoroShiro {
    const XL: u64 = 0x9e3779b97f4a7c15;
    const XH: u64 = 0x6a09e667f3bcc909;
    const A: u64 = 0xbf58476d1ce4e5b9;
    const B: u64 = 0x94d049bb133111eb;

    pub fn new(seed: u64) -> Self {
        let mut l = seed ^ Self::XH;
        let mut h = l.wrapping_add(Self::XL);
        l = (l ^ (l >> 30)).wrapping_mul(Self::A);
        h = (h ^ (h >> 30)).wrapping_mul(Self::A);
        l = (l ^ (l >> 27)).wrapping_mul(Self::B);
        h = (h ^ (h >> 27)).wrapping_mul(Self::B);
        l = l ^ (l >> 31);
        h = h ^ (h >> 31);

        Self {
            lo: l,
            hi: h,
        }
    }

    pub fn from_lo_hi(lo: u64, hi: u64) -> Self {
        Self {
            lo,
            hi,
        }
    }
}

impl Rng for XoroShiro {
    fn next_u64(&mut self) -> u64 {
        let l = self.lo;
        let mut h = self.hi;
        let n = l.wrapping_add(h).rotate_left(17).wrapping_add(l);
        h ^= l;
        self.lo = l.rotate_left(49) ^ h ^ (h << 21);
        self.hi = h.rotate_left(28);

        n
    }

    fn next_u32(&mut self, max_value: u32) -> u32 {
        let max_value = max_value as u64;
        let mut r = (self.next_u64() & 0xFFFFFFFF).wrapping_mul(max_value);
        if r < max_value {
            while r < (!max_value + 1) % max_value {
                r = (self.next_u64() & 0xFFFFFFFF).wrapping_mul(max_value);
            }
        }

        (r >> 32) as u32
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> (64 - 53)) as f64 * 1.1102230246251565E-16
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> (64 - 24)) as f32 * 5.9604645E-8
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 != 0
    }
}

impl RngForkable for XoroShiro {
    type Pos = XoroShiroPos;

    fn fork_pos(&mut self) -> Self::Pos {
        Self::Pos::from_rng(self)
    }
}

pub struct XoroShiroPos {
    lo: u64,
    hi: u64,
}

impl XoroShiroPos {
    #[inline]
    pub fn new(lo: u64, hi: u64) -> Self {
        Self {
            lo,
            hi,
        }
    }
}

impl RngPos for XoroShiroPos {
    type Item = XoroShiro;

    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        Self::new(rng.next_u64(), rng.next_u64())
    }

    fn at(&self, pos: Vector3) -> Self::Item {
        let i = pos.seed() as u64;
        let j = i ^ self.lo;

        Self::Item::from_lo_hi(j, self.hi)
    }

    fn by_hash(&self, s: String) -> Self::Item {
        let digest = md5::compute(s);

        let bytes = digest.as_slice();
        let lo = u64::from_be_bytes((&bytes[..8]).try_into().unwrap());
        let hi = u64::from_be_bytes((&bytes[8..]).try_into().unwrap());

        Self::Item::from_lo_hi(self.lo ^ lo, self.hi ^ hi)
    }
}

/// Trait defining an entropy source that provides 32-bit unsigned integers.  This is
/// an abstraction for different ways to generate randomness.
pub trait U32EntropySrc {
    /// Creates a new entropy source with the given seed. The seed is used to initialize
    /// the internal state of the entropy source.
    fn new(seed: u64) -> Self;

    /// Generates the next `bits` bits of random data.  The number of bits requested must
    /// be less than or equal to 32.
    fn next(&mut self, bits: u32) -> u32;
}

/// Minecraft-specific implementation of [`U32EntropySrc`] trait for XoroShiro algo.
pub struct XoroShiroU32EntropySrc(XoroShiro);

impl XoroShiroU32EntropySrc {
    #[inline]
    pub fn new(xoro: XoroShiro) -> Self {
        Self(xoro)
    }
}

impl U32EntropySrc for XoroShiroU32EntropySrc {
    fn new(seed: u64) -> Self {
        Self(XoroShiro::new(seed))
    }

    fn next(&mut self, bits: u32) -> u32 {
        (self.0.next_u64() >> 64 - bits) as u32
    }
}

/// A simple [`Linear Congruential Generator`] (LCG) for generating pseudo-random numbers.
///
/// It uses formula `X_(n+1) = (a * X_n + c) mod m`
/// with the following values of the parameters:
/// ```text
/// a: 25214903917
/// c: 11
/// m: 2^48 - 1
/// ```
///
/// This generator is mostly outdated and is only used in some parts of the world generator.
/// The rest of the code uses [`XoroShiro`].
///
/// [`Linear Congruential Generator`]: https://en.wikipedia.org/wiki/Linear_congruential_generator
pub struct LcgEntropySrc {
    state: u64,
}

impl LcgEntropySrc {
    const MULTIPLIER: u64 = 25214903917;
    const INCREMENT: u64 = 11;
    const MOD_BITS: u32 = 48;
    const MOD_MASK: u64 = 2u64.pow(Self::MOD_BITS) - 1;
}

impl U32EntropySrc for LcgEntropySrc {
    #[inline]
    fn new(seed: u64) -> Self {
        Self {
            state: (seed ^ 25214903917) & 281474976710655,
        }
    }

    #[inline]
    fn next(&mut self, bits: u32) -> u32 {
        let state = (self.state.wrapping_mul(Self::MULTIPLIER) + Self::INCREMENT) & Self::MOD_MASK;
        self.state = state;

        (state >> (48 - bits)) as u32
    }
}

/// Random number generator that uses [`U32EntropySrc`] as an underlying entropy source.
pub struct U32EntropySrcRng<S>
where
    S: U32EntropySrc
{
    /// The underlying entropy source. This is the core source of randomness for this RNG.
    src: S
}

impl<S> U32EntropySrcRng<S>
where
    S: U32EntropySrc
{
    #[inline]
    pub fn new(src: S) -> Self {
        Self {
            src
        }
    }
}

impl<S> Rng for U32EntropySrcRng<S>
where
    S: U32EntropySrc
{
    fn next_u64(&mut self) -> u64 {
        let i = self.src.next(32) as i32;
        let j = self.src.next(32) as i32;
        let k = (i as i64) << 32;

        (k + j as i64) as u64
    }

    fn next_u32(&mut self, max_value: u32) -> u32 {
        if max_value == 0 {
            panic!("max_value must be greater than 0");
        } else if is_power_of_two(max_value) {
            (max_value as u64 * self.src.next(31) as u64 >> 31) as u32
        } else {
            let max_value = max_value as i32;

            let mut i;
            let mut j;

            // do-while loop
            while {
                i = self.src.next(31) as i32;
                j = i % max_value;

                i - j + (max_value - 1) < 0
            } {}

            j as u32
        }
    }

    fn next_f64(&mut self) -> f64 {
        let i = self.src.next(26) as u64;
        let j = self.src.next(27) as u64;
        let k = (i << 27) + j;

        k as f64 * 1.110223E-16
    }

    fn next_f32(&mut self) -> f32 {
        self.src.next(24) as f32 * 5.9604645E-8
    }

    fn next_bool(&mut self) -> bool {
        self.src.next(1) != 0
    }
}

impl RngForkable for U32EntropySrcRng<XoroShiroU32EntropySrc> {
    type Pos = XoroShiroPos;

    fn fork_pos(&mut self) -> Self::Pos {
        XoroShiroPos::new(self.src.0.next_u64(), self.src.0.next_u64())
    }
}

impl RngForkable for U32EntropySrcRng<LcgEntropySrc> {
    type Pos = U32EntropySrcRngPos;

    fn fork_pos(&mut self) -> Self::Pos {
        U32EntropySrcRngPos::new(self.next_u64())
    }
}

pub struct U32EntropySrcRngPos {
    state: u64
}

/// [`RngPos`] that is yielded by [`U32EntropySrcRng`].
impl U32EntropySrcRngPos
{
    pub fn new(state: u64) -> Self {
        Self {
            state
        }
    }
}

impl RngPos for U32EntropySrcRngPos
{
    type Item = U32EntropySrcRng<LcgEntropySrc>;

    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        Self::new(rng.next_u64())
    }

    fn at(&self, pos: Vector3) -> Self::Item {
        Self::Item::new(LcgEntropySrc::new(pos.seed() as u64 ^ self.state))
    }

    fn by_hash(&self, s: String) -> Self::Item {
        let mut hasher = JavaStringHasher::default();
        hasher.write(s.as_bytes());

        Self::Item::new(LcgEntropySrc::new(hasher.finish() ^ self.state))
    }
}

/// A custom hasher that mimics the behavior of Java's `String.hashCode()` for ASCII strings.
/// More precisely, it behaves like `StringLatin1.hashCode()` method.
/// This implementation uses a simple multiplicative hash function with a multiplier of 31.
///
/// Note: This hasher only handles ASCII characters correctly.  Non-ASCII characters will
/// be treated as bytes.
#[derive(Default)]
struct JavaStringHasher {
    state: i32,
}

impl Hasher for JavaStringHasher {
    fn finish(&self) -> u64 {
        self.state as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut h = 0;
        for b in bytes {
            h = 31i32.wrapping_mul(h).wrapping_add(*b as i32);
        }

        self.state = h;
    }
}

#[cfg(test)]
mod tests {
    use crate::rng::{JavaStringHasher, LcgEntropySrc, Rng, RngForkable, RngPos, U32EntropySrc, U32EntropySrcRng, XoroShiro, XoroShiroPos, XoroShiroU32EntropySrc};
    use spherix_math::vector::Vector3;
    use spherix_util::{assert_f32_eq, assert_f64_eq};
    use std::hash::Hasher;

    #[test]
    fn xoroshiro_new() {
        let xoro = XoroShiro::new(1);
        assert_eq!(5272463233947570727i64 as u64, xoro.lo);
        assert_eq!(1927618558350093866i64 as u64, xoro.hi);
    }

    #[test]
    fn xoroshiro_fork_pos() {
        let mut xoro = XoroShiro::new(1);
        let pos = xoro.fork_pos();
        assert_eq!(-1033667707219518978i64 as u64, pos.lo);
        assert_eq!(6451672561743293322i64 as u64, pos.hi);
    }

    #[test]
    fn xoroshiro_next_u64() {
        let mut xoro = XoroShiro::from_lo_hi(2, 1);
        assert_eq!(0x60002, xoro.next_u64());
        assert_eq!(0x460C00066000B, xoro.next_u64());
        assert_eq!(0x1814CC08B18606D7, xoro.next_u64());

        let mut xoro = XoroShiro::from_lo_hi(0x2491583821242122, 0x4928100127503943);
        assert_eq!(0xF503E920D5EEFC94, xoro.next_u64());
        assert_eq!(0x4B1B7198977DCCB3, xoro.next_u64());

        let mut xoro = XoroShiro::new(0xAA26BD80C479);
        assert_eq!(0xCA6831A66A4D55FC, xoro.next_u64());
        assert_eq!(0xCFB77C6B4387C3EF, xoro.next_u64());
        assert_eq!(0xF938664618966150, xoro.next_u64());
    }

    #[test]
    fn xoroshiro_next_f64() {
        let mut xoro = XoroShiro::new(0x3893D182AC120);
        assert_f64_eq!(0.344070, xoro.next_f64(), 5);
        assert_f64_eq!(0.473408, xoro.next_f64(), 5);
        assert_f64_eq!(0.463280, xoro.next_f64(), 5);
        assert_f64_eq!(0.819293, xoro.next_f64(), 5);
        assert_f64_eq!(0.165539, xoro.next_f64(), 5);
    }

    #[test]
    fn xoroshiro_next_u32() {
        let mut xoro = XoroShiro::new(0xE3875F0528FA);
        assert_eq!(55, xoro.next_u32(64));
        assert_eq!(32, xoro.next_u32(64));
        assert_eq!(1, xoro.next_u32(32));
        assert_eq!(36, xoro.next_u32(64));
        assert_eq!(65, xoro.next_u32(128));

        let mut xoro = XoroShiro::new(0x7FB09B7CCD6F0BB4);
        assert_eq!(2138517, xoro.next_u32(5000000));
        assert_eq!(1790114, xoro.next_u32(10000000));
        assert_eq!(2793229, xoro.next_u32(7500000));
        assert_eq!(5935974, xoro.next_u32(12500000));
        assert_eq!(1864049, xoro.next_u32(10000000));
    }

    #[test]
    fn xoroshiro_next_bool() {
        let mut xoro = XoroShiro::new(0xDFFF71);
        assert_eq!(false, xoro.next_bool());
        assert_eq!(false, xoro.next_bool());
        assert_eq!(true, xoro.next_bool());
        assert_eq!(false, xoro.next_bool());
        assert_eq!(false, xoro.next_bool());
        assert_eq!(true, xoro.next_bool());
        assert_eq!(false, xoro.next_bool());
        assert_eq!(true, xoro.next_bool());
        assert_eq!(true, xoro.next_bool());
        assert_eq!(false, xoro.next_bool());
    }
    
    #[test]
    fn xoroshiro_next_u32_inclusive() {
        let mut xoro = XoroShiro::new(0x12A00C473C0);
        assert_eq!(108, xoro.next_u32_inclusive(24, 108));
        assert_eq!(462, xoro.next_u32_inclusive(0, 500));
        assert_eq!(498, xoro.next_u32_inclusive(256, 1024));
    }

    #[test]
    fn xoroshiro_pos() {
        let mut xoro = XoroShiro::new(0x9B9B46C40A);
        let pos = xoro.fork_pos();
        assert_eq!(5394267108863772786, pos.lo);
        assert_eq!(-8469858705098465684i64 as u64, pos.hi);

        let mut pos_at = pos.at(Vector3::new(20, -10, 9512));
        assert_eq!(5394264719504356683, pos_at.lo);
        assert_eq!(-8469858705098465684i64 as u64, pos_at.hi);

        assert_eq!(-2162372296719048723i64 as u64, pos_at.next_u64());
    }

    #[test]
    fn lcg_next() {
        let mut rng = LcgEntropySrc::new(0xC35AA338);
        assert_eq!(21973321045, rng.state);
        assert_eq!(422, rng.next(10));
        assert_eq!(116204790473532, rng.state);
        assert_eq!(1144825477, rng.next(32));
        assert_eq!(75027282503831, rng.state);
    }

    #[test]
    fn lcg_next_u32() {
        let mut rng = U32EntropySrcRng::new(LcgEntropySrc::new(0xC35AA338));
        assert_eq!(2, rng.next_u32(10));
        assert_eq!(12738, rng.next_u32(25000));
        assert_eq!(65, rng.next_u32(256));
        assert_eq!(160, rng.next_u32(1024));
    }

    #[test]
    fn lcg_next_u64() {
        let mut rng = U32EntropySrcRng::new(LcgEntropySrc::new(0x70263B5E));
        assert_eq!(9100319232341836070, rng.next_u64());
        assert_eq!(-7781228307940896489i64 as u64, rng.next_u64());
        assert_eq!(4672688588979360656, rng.next_u64());
        assert_eq!(631219011787583875, rng.next_u64());
        assert_eq!(-6356805321547395082i64 as u64, rng.next_u64());
    }

    #[test]
    fn lcg_next_f32() {
        let mut rng = U32EntropySrcRng::new(LcgEntropySrc::new(0xB58DA950));
        assert_f32_eq!(0.80232376, rng.next_f32(), 5);
        assert_f32_eq!(0.3730473, rng.next_f32(), 5);
        assert_f32_eq!(0.80555546, rng.next_f32(), 5);
        assert_f32_eq!(0.5311839, rng.next_f32(), 5);
        assert_f32_eq!(0.87817776, rng.next_f32(), 5);
    }

    #[test]
    fn lcg_next_f64() {
        let mut rng = U32EntropySrcRng::new(LcgEntropySrc::new(0x44749BA8));

        assert_f64_eq!(0.2569215006562042, rng.next_f64(), 7);
        assert_f64_eq!(0.7842214799296693, rng.next_f64(), 7);
        assert_f64_eq!(0.6982273956897361, rng.next_f64(), 7);
        assert_f64_eq!(0.9415300553099406, rng.next_f64(), 7);
        assert_f64_eq!(0.9148220775552723, rng.next_f64(), 7);
    }

    #[test]
    fn lcg_next_bool() {
        let mut rng = U32EntropySrcRng::new(LcgEntropySrc::new(0xDFFF71));
        assert_eq!(true, rng.next_bool());
        assert_eq!(false, rng.next_bool());
        assert_eq!(false, rng.next_bool());
        assert_eq!(true, rng.next_bool());
        assert_eq!(false, rng.next_bool());
        assert_eq!(true, rng.next_bool());
        assert_eq!(false, rng.next_bool());
        assert_eq!(true, rng.next_bool());
        assert_eq!(false, rng.next_bool());
        assert_eq!(true, rng.next_bool());
    }

    #[test]
    fn lcg_next_u32_inclusive() {
        let mut xoro = U32EntropySrcRng::new(LcgEntropySrc::new(0x12A00C473C0));
        assert_eq!(104, xoro.next_u32_inclusive(24, 108));
        assert_eq!(89, xoro.next_u32_inclusive(0, 500));
        assert_eq!(517, xoro.next_u32_inclusive(256, 1024));
    }
    
    #[test]
    fn lcg_pos_at() {
        let mut rng = U32EntropySrcRng::new(LcgEntropySrc::new(0xDF64D427));
        let pos = rng.fork_pos();
        let rng_at = pos.at(Vector3::new(41, 24, -7));
        assert_eq!(99938300968846, rng_at.src.state);

        let rng_at = pos.at(Vector3::new(-24015, 10007, 58275));
        assert_eq!(66595076963162, rng_at.src.state);

        let rng_at = pos.at(Vector3::new(-23985832, -3925385, -1284425));
        assert_eq!(61214541779895, rng_at.src.state);
    }

    #[test]
    fn java_string_hasher() {
        let mut hasher = JavaStringHasher::default();
        hasher.write("minecraft:test".as_bytes());
        assert_eq!(-1006394817i64 as u64, hasher.finish());

        let mut hasher = JavaStringHasher::default();
        hasher.write(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
            incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
            exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute \
            irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla \
            pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia \
            deserunt mollit anim id est laborum."
                .as_bytes()
        );
        assert_eq!(512895612, hasher.finish());
    }

    #[test]
    fn lcg_pos_by_hash() {
        let mut rng = U32EntropySrcRng::new(LcgEntropySrc::new(0xDF64D427));
        let pos = rng.fork_pos();
        let rng_by_hash = pos.by_hash("minecraft:test".to_owned());
        assert_eq!(127100847861395, rng_by_hash.src.state);

        let rng_by_hash = pos.by_hash("minecraft:random".to_owned());
        assert_eq!(127100717565852, rng_by_hash.src.state);

        let rng_by_hash = pos.by_hash(
            "This function will never receive such a long string in a real-world call".to_owned()
        );
        assert_eq!(127100881780935, rng_by_hash.src.state);
    }

    #[test]
    fn entropy_source_xoroshiro_next() {
        let mut rng = XoroShiroU32EntropySrc::new(XoroShiro::new(0xCB50BB6F));
        assert_eq!(810, rng.next(10));
        assert_eq!((-8350838413437023994i64 as u64, 975853854573731968), (rng.0.lo, rng.0.hi));
        assert_eq!(559435425, rng.next(32));
        assert_eq!((4042745542287828298, -4373163908385926379i64 as u64), (rng.0.lo, rng.0.hi));
    }


    #[test]
    fn entropy_source_xoroshiro_rng_next_u32() {
        let mut rng = U32EntropySrcRng::new(XoroShiroU32EntropySrc::new(XoroShiro::new(0x9C439A7D)));

        assert_eq!(0, rng.next_u32(10));
        assert_eq!(10232, rng.next_u32(25000));
        assert_eq!(47, rng.next_u32(256));
        assert_eq!(243, rng.next_u32(1024));
    }

    #[test]
    fn entropy_source_xoroshiro_rng_next_u64() {
        let mut rng = U32EntropySrcRng::new(XoroShiroU32EntropySrc::new(XoroShiro::new(0x77EA110A)));

        assert_eq!(7407654806463502031, rng.next_u64());
        assert_eq!(6464672045709718130, rng.next_u64());
        assert_eq!(7146482241115983707, rng.next_u64());
        assert_eq!(173546863784073648, rng.next_u64());
        assert_eq!(-1908722545387692185i64 as u64, rng.next_u64());
    }

    #[test]
    fn entropy_source_xoroshiro_rng_next_f32() {
        let mut rng = U32EntropySrcRng::new(XoroShiroU32EntropySrc::new(XoroShiro::new(0xF23E9F6F)));

        assert_f32_eq!(0.36856675, rng.next_f32(), 5);
        assert_f32_eq!(0.658037, rng.next_f32(), 5);
        assert_f32_eq!(0.59937584, rng.next_f32(), 5);
        assert_f32_eq!(0.90693426, rng.next_f32(), 5);
        assert_f32_eq!(0.78840834, rng.next_f32(), 5);
    }

    #[test]
    fn entropy_source_xoroshiro_rng_next_f64() {
        let mut rng = U32EntropySrcRng::new(XoroShiroU32EntropySrc::new(XoroShiro::new(0x325B29CF)));

        assert_f64_eq!(0.3224586326038592, rng.next_f64(), 7);
        assert_f64_eq!(0.8100203529515602, rng.next_f64(), 7);
        assert_f64_eq!(0.9609253188880038, rng.next_f64(), 7);
        assert_f64_eq!(0.21963313061691148, rng.next_f64(), 7);
        assert_f64_eq!(0.3723346936372929, rng.next_f64(), 7);
    }

    #[test]
    fn entropy_source_xoroshiro_rng_pos_at() {
        let mut rng = U32EntropySrcRng::new(XoroShiroU32EntropySrc::new(XoroShiro::new(0x5B584A8E)));
        let pos: XoroShiroPos = rng.fork_pos();
        let rng_at = pos.at(Vector3::new(41, 24, -7));
        assert_eq!((5935213701552871892, -7347259573176361533i64 as u64), (rng_at.lo, rng_at.hi));

        let rng_at = pos.at(Vector3::new(-24015, 10007, 58275));
        assert_eq!((5935316735050472704, -7347259573176361533i64 as u64), (rng_at.lo, rng_at.hi));

        let rng_at = pos.at(Vector3::new(-23985832, -3925385, -1284425));
        assert_eq!((5935304652248462829, -7347259573176361533i64 as u64), (rng_at.lo, rng_at.hi));
    }
}
