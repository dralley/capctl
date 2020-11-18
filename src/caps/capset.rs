use std::fmt;
use std::iter::FromIterator;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign,
};

use super::{Cap, CAP_BITMASK, NUM_CAPS};

/// Represents a set of capabilities.
///
/// Internally, this stores the set of capabilities as a bitmask, which is much more efficient than
/// a `HashSet<Cap>`.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct CapSet {
    pub(super) bits: u64,
}

impl CapSet {
    /// Create an empty capability set.
    #[inline]
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Clear all capabilities from this set.
    ///
    /// After this call, `set.is_empty()` will return `true`.
    #[inline]
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    /// Check if this capability set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Returns the number of capabilities in this capability set.
    #[inline]
    pub fn size(&self) -> usize {
        self.bits.count_ones() as usize
    }

    /// Checks if a given capability is present in this set.
    #[inline]
    pub fn has(&self, cap: Cap) -> bool {
        self.bits & cap.to_single_bitfield() != 0
    }

    /// Adds the given capability to this set.
    #[inline]
    pub fn add(&mut self, cap: Cap) {
        self.bits |= cap.to_single_bitfield();
    }

    /// Removes the given capability from this set.
    #[inline]
    pub fn drop(&mut self, cap: Cap) {
        self.bits &= !cap.to_single_bitfield();
    }

    /// If `val` is `true` the given capability is added; otherwise it is removed.
    pub fn set_state(&mut self, cap: Cap, val: bool) {
        if val {
            self.add(cap);
        } else {
            self.drop(cap);
        }
    }

    /// Adds all of the capabilities yielded by the given iterator to this set.
    ///
    /// If you want to add all the capabilities in another capability set, you should use
    /// `set1 = set1.union(set2)` or `set1 = set1 | set2`, NOT `set1.add_all(set2)`.
    pub fn add_all<T: IntoIterator<Item = Cap>>(&mut self, t: T) {
        for cap in t.into_iter() {
            self.add(cap);
        }
    }

    /// Removes all of the capabilities yielded by the given iterator from this set.
    ///
    /// If you want to remove all the capabilities in another capability set, you should use
    /// `set1 = set1.intersection(!set2)` or `set1 = set1 & !set2`, NOT `set1.drop_all(set2)`.
    pub fn drop_all<T: IntoIterator<Item = Cap>>(&mut self, t: T) {
        for cap in t.into_iter() {
            self.drop(cap);
        }
    }

    /// Returns an iterator over all of the capabilities in this set.
    #[inline]
    pub fn iter(&self) -> CapSetIterator {
        self.into_iter()
    }

    /// Returns the union of this set and another capability set (i.e. all the capabilities that
    /// are in either set).
    #[inline]
    pub const fn union(&self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// Returns the intersection of this set and another capability set (i.e. all the capabilities
    /// that are in both sets).
    #[inline]
    pub const fn intersection(&self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }

    /// WARNING: This is an internal method and its signature may change in the future. Use [the
    /// `capset!()` macro] instead.
    ///
    /// [the `capset!()` macro]: ../macro.capset.html
    #[doc(hidden)]
    #[inline]
    pub fn from_bitmask_truncate(bitmask: u64) -> Self {
        Self {
            bits: bitmask & CAP_BITMASK,
        }
    }

    #[inline]
    pub(crate) fn from_bitmasks_u32(lower: u32, upper: u32) -> Self {
        Self::from_bitmask_truncate(((upper as u64) << 32) | (lower as u64))
    }
}

impl Default for CapSet {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl Not for CapSet {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self {
            bits: (!self.bits) & CAP_BITMASK,
        }
    }
}

impl BitAnd for CapSet {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl BitOr for CapSet {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitXor for CapSet {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits ^ rhs.bits,
        }
    }
}

impl Sub for CapSet {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & (!rhs.bits),
        }
    }
}

impl BitAndAssign for CapSet {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOrAssign for CapSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for CapSet {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl SubAssign for CapSet {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Extend<Cap> for CapSet {
    #[inline]
    fn extend<I: IntoIterator<Item = Cap>>(&mut self, it: I) {
        self.add_all(it);
    }
}

impl FromIterator<Cap> for CapSet {
    #[inline]
    fn from_iter<I: IntoIterator<Item = Cap>>(it: I) -> Self {
        let mut res = Self::empty();
        res.extend(it);
        res
    }
}

impl IntoIterator for CapSet {
    type Item = Cap;
    type IntoIter = CapSetIterator;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CapSetIterator { set: self, i: 0 }
    }
}

impl fmt::Debug for CapSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// A helper macro to statically construct a `CapSet` from a list of capabilities.
///
/// Examples:
/// ```
/// use std::iter::FromIterator;
/// use capctl::capset;
/// use capctl::caps::{Cap, CapSet};
///
/// assert_eq!(capset!(), CapSet::empty());
/// assert_eq!(capset!(Cap::CHOWN), CapSet::from_iter(vec![Cap::CHOWN]));
/// assert_eq!(capset!(Cap::CHOWN, Cap::SYSLOG), CapSet::from_iter(vec![Cap::CHOWN, Cap::SYSLOG]));
/// ```
#[macro_export]
macro_rules! capset {
    () => {
        $crate::caps::CapSet::empty()
    };
    ($cap:expr$(, $caps:expr)*) => {
        $crate::caps::CapSet::from_bitmask_truncate((1 << ($cap as $crate::caps::Cap as u8)) $(| (1 << ($caps as $crate::caps::Cap as u8)))*)
    };
    ($cap:expr, $($caps:expr,)*) => {
        capset!($cap$(, $caps)*)
    };
}

/// An iterator over all the capabilities in a `CapSet`.
///
/// This is constructed by [`CapSet::iter()`].
///
/// [`CapSet::iter()`]: ./struct.CapSet.html#method.iter
#[derive(Clone)]
pub struct CapSetIterator {
    set: CapSet,
    i: u8,
}

impl Iterator for CapSetIterator {
    type Item = Cap;

    fn next(&mut self) -> Option<Cap> {
        while let Some(cap) = Cap::from_u8(self.i) {
            self.i += 1;
            if self.set.has(cap) {
                return Some(cap);
            }
        }

        None
    }

    #[inline]
    fn last(self) -> Option<Cap> {
        // This calculates the position of the largest bit that is set.
        // For example, if the bitmask is 0b10101, n=5.
        let n = std::mem::size_of::<u64>() as u8 * 8 - self.set.bits.leading_zeros() as u8;

        if self.i < n {
            // We haven't yet passed the largest bit.
            // This uses `<` instead of `<=` because `self.i` and `n` are off by 1 (so we also have
            // to subtract 1 below).

            let res = Cap::from_u8(n - 1);
            debug_assert!(res.is_some());
            res
        } else {
            None
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for CapSetIterator {
    #[inline]
    fn len(&self) -> usize {
        // It should be literally impossible for i to be out of this range
        debug_assert!(self.i <= NUM_CAPS);

        (self.set.bits >> self.i).count_ones() as usize
    }
}

impl std::iter::FusedIterator for CapSetIterator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capset_empty() {
        let mut set = CapSet::from_iter(Cap::iter());
        for cap in Cap::iter() {
            set.drop(cap);
        }
        assert_eq!(set.bits, 0);
        assert!(set.is_empty());

        set = CapSet::empty();
        assert_eq!(set.bits, 0);
        assert!(set.is_empty());
        assert_eq!(set, CapSet::default());

        set = CapSet::from_iter(Cap::iter());
        set.clear();
        assert_eq!(set.bits, 0);
        assert!(set.is_empty());

        assert!(!Cap::iter().any(|c| set.has(c)));
    }

    #[test]
    fn test_capset_full() {
        let mut set = CapSet::empty();
        for cap in Cap::iter() {
            set.add(cap);
        }
        assert_eq!(set.bits, CAP_BITMASK);
        assert!(!set.is_empty());

        set = CapSet::empty();
        set.extend(Cap::iter());
        assert_eq!(set.bits, CAP_BITMASK);
        assert!(!set.is_empty());

        assert!(Cap::iter().all(|c| set.has(c)));
    }

    #[test]
    fn test_capset_add_drop() {
        let mut set = CapSet::empty();
        set.add(Cap::CHOWN);
        assert!(set.has(Cap::CHOWN));
        assert!(!set.is_empty());

        set.drop(Cap::CHOWN);
        assert!(!set.has(Cap::CHOWN));
        assert!(set.is_empty());

        set.set_state(Cap::CHOWN, true);
        assert!(set.has(Cap::CHOWN));
        assert!(!set.is_empty());

        set.set_state(Cap::CHOWN, false);
        assert!(!set.has(Cap::CHOWN));
        assert!(set.is_empty());
    }

    #[test]
    fn test_capset_add_drop_all() {
        let mut set = CapSet::empty();
        set.add_all(vec![Cap::FOWNER, Cap::CHOWN, Cap::KILL]);

        // Iteration order is not preserved, but it should be consistent.
        assert_eq!(
            set.into_iter().collect::<Vec<Cap>>(),
            vec![Cap::CHOWN, Cap::FOWNER, Cap::KILL]
        );
        assert_eq!(
            set.iter().collect::<Vec<Cap>>(),
            vec![Cap::CHOWN, Cap::FOWNER, Cap::KILL]
        );

        set.drop_all(vec![Cap::FOWNER, Cap::CHOWN]);
        assert_eq!(set.iter().collect::<Vec<Cap>>(), vec![Cap::KILL]);

        set.drop_all(vec![Cap::KILL]);
        assert_eq!(set.iter().collect::<Vec<Cap>>(), vec![]);
    }

    #[test]
    fn test_capset_from_iter() {
        let set = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]);
        assert_eq!(
            set.iter().collect::<Vec<Cap>>(),
            vec![Cap::CHOWN, Cap::FOWNER],
        );
    }

    #[test]
    fn test_capset_iter_full() {
        assert!(Cap::iter().eq(CapSet { bits: CAP_BITMASK }.iter()));
        assert!(Cap::iter().eq(CapSet::from_iter(Cap::iter()).iter()));
    }

    #[test]
    fn test_capset_iter_count() {
        for set in [
            CapSet::from_iter(vec![]),
            CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]),
            CapSet::from_iter(Cap::iter()),
        ]
        .iter()
        {
            let mut count = set.size();

            let mut it = set.iter();
            assert_eq!(it.len(), count);
            assert_eq!(it.clone().count(), count);
            assert_eq!(it.size_hint(), (count, Some(count)));

            while let Some(_cap) = it.next() {
                count -= 1;
                assert_eq!(it.len(), count);
                assert_eq!(it.clone().count(), count);
                assert_eq!(it.size_hint(), (count, Some(count)));
            }

            assert_eq!(count, 0);

            assert_eq!(it.len(), 0);
            assert_eq!(it.clone().count(), 0);
            assert_eq!(it.size_hint(), (0, Some(0)));
        }
    }

    #[test]
    fn test_capset_iter_last() {
        let last_cap = Cap::iter().last().unwrap();

        assert_eq!(CapSet::from_iter(Cap::iter()).iter().last(), Some(last_cap));
        assert_eq!(CapSet::empty().iter().last(), None);

        let mut it = CapSet::from_iter(Cap::iter()).iter();
        assert_eq!(it.clone().last(), Some(last_cap));
        while it.next().is_some() {
            if it.clone().next().is_some() {
                assert_eq!(it.clone().last(), Some(last_cap));
            } else {
                assert_eq!(it.clone().last(), None);
            }
        }
        assert_eq!(it.len(), 0);
        assert_eq!(it.last(), None);

        it = capset!(Cap::FOWNER).iter();
        assert_eq!(it.clone().last(), Some(Cap::FOWNER));
        assert_eq!(it.next(), Some(Cap::FOWNER));
        assert_eq!(it.last(), None);

        it = capset!(Cap::CHOWN).iter();
        assert_eq!(it.clone().last(), Some(Cap::CHOWN));
        assert_eq!(it.next(), Some(Cap::CHOWN));
        assert_eq!(it.last(), None);

        it = capset!(Cap::CHOWN, Cap::FOWNER).iter();
        assert_eq!(it.clone().last(), Some(Cap::FOWNER));
        assert_eq!(it.next(), Some(Cap::CHOWN));
        assert_eq!(it.clone().last(), Some(Cap::FOWNER));
        assert_eq!(it.next(), Some(Cap::FOWNER));
        assert_eq!(it.clone().last(), None);
        assert_eq!(it.next(), None);
        assert_eq!(it.last(), None);
    }

    #[test]
    fn test_capset_union() {
        let a = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]);
        let b = CapSet::from_iter(vec![Cap::FOWNER, Cap::KILL]);
        let c = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER, Cap::KILL]);
        assert_eq!(a.union(b), c);
    }

    #[test]
    fn test_capset_intersection() {
        let a = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]);
        let b = CapSet::from_iter(vec![Cap::FOWNER, Cap::KILL]);
        let c = CapSet::from_iter(vec![Cap::FOWNER]);
        assert_eq!(a.intersection(b), c);
    }

    #[test]
    fn test_capset_not() {
        assert_eq!(!CapSet::from_iter(Cap::iter()), CapSet::empty());
        assert_eq!(CapSet::from_iter(Cap::iter()), !CapSet::empty());

        let mut a = CapSet::from_iter(Cap::iter());
        let mut b = CapSet::empty();
        a.add(Cap::CHOWN);
        b.drop(Cap::CHOWN);
        assert_eq!(!a, b);
    }

    #[test]
    fn test_capset_bitor() {
        let a = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]);
        let b = CapSet::from_iter(vec![Cap::FOWNER, Cap::KILL]);
        let c = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER, Cap::KILL]);
        assert_eq!(a | b, c);

        let mut d = a;
        d |= b;
        assert_eq!(d, c);
    }

    #[test]
    fn test_capset_bitand() {
        let a = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]);
        let b = CapSet::from_iter(vec![Cap::FOWNER, Cap::KILL]);
        let c = CapSet::from_iter(vec![Cap::FOWNER]);
        assert_eq!(a & b, c);

        let mut d = a;
        d &= b;
        assert_eq!(d, c);
    }

    #[test]
    fn test_capset_bitxor() {
        let a = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]);
        let b = CapSet::from_iter(vec![Cap::FOWNER, Cap::KILL]);
        let c = CapSet::from_iter(vec![Cap::CHOWN, Cap::KILL]);
        assert_eq!(a ^ b, c);

        let mut d = a;
        d ^= b;
        assert_eq!(d, c);
    }

    #[test]
    fn test_capset_sub() {
        let a = CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER]);
        let b = CapSet::from_iter(vec![Cap::FOWNER, Cap::KILL]);
        let c = CapSet::from_iter(vec![Cap::CHOWN]);
        assert_eq!(a - b, c);

        let mut d = a;
        d -= b;
        assert_eq!(d, c);
    }

    #[test]
    fn test_capset_fmt() {
        assert_eq!(format!("{:?}", CapSet::empty()), "{}");
        assert_eq!(
            format!("{:?}", CapSet::from_iter(vec![Cap::CHOWN])),
            "{CHOWN}"
        );
        assert_eq!(
            format!("{:?}", CapSet::from_iter(vec![Cap::CHOWN, Cap::FOWNER])),
            "{CHOWN, FOWNER}"
        );
    }

    #[test]
    fn test_capset_macro() {
        assert_eq!(capset!(), CapSet::empty());

        assert_eq!(capset!(Cap::CHOWN), CapSet::from_iter(vec![Cap::CHOWN]));
        assert_eq!(capset!(Cap::CHOWN,), CapSet::from_iter(vec![Cap::CHOWN]));

        let cap = Cap::CHOWN;
        assert_eq!(capset!(cap), CapSet::from_iter(vec![cap]));
        assert_eq!(capset!(cap,), CapSet::from_iter(vec![cap]));

        assert_eq!(
            capset!(Cap::CHOWN, Cap::SYSLOG),
            CapSet::from_iter(vec![Cap::CHOWN, Cap::SYSLOG])
        );
        assert_eq!(
            capset!(Cap::CHOWN, Cap::SYSLOG,),
            CapSet::from_iter(vec![Cap::CHOWN, Cap::SYSLOG])
        );

        assert_eq!(
            capset!(Cap::CHOWN, Cap::SYSLOG, Cap::FOWNER),
            CapSet::from_iter(vec![Cap::CHOWN, Cap::SYSLOG, Cap::FOWNER])
        );
        assert_eq!(
            capset!(Cap::CHOWN, Cap::SYSLOG, Cap::FOWNER,),
            CapSet::from_iter(vec![Cap::CHOWN, Cap::SYSLOG, Cap::FOWNER])
        );
    }
}
