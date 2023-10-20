use crate::simd::intrinsics;
use crate::simd::{LaneCount, Mask, MaskElement, Simd, SimdElement, SupportedLaneCount};

/// Constructs a new SIMD vector by copying elements from selected elements in other vectors.
///
/// When swizzling one vector, elements are selected like [`Swizzle::swizzle`].
///
/// When swizzling two vectors, elements are selected like [`Swizzle::concat_swizzle`].
///
/// # Examples
///
/// With a single SIMD vector, the const array specifies element indices in that vector:
/// ```
/// # #![feature(portable_simd)]
/// # use core::simd::{u32x2, u32x4, simd_swizzle};
/// let v = u32x4::from_array([10, 11, 12, 13]);
///
/// // Keeping the same size
/// let r: u32x4 = simd_swizzle!(v, [3, 0, 1, 2]);
/// assert_eq!(r.to_array(), [13, 10, 11, 12]);
///
/// // Changing the number of elements
/// let r: u32x2 = simd_swizzle!(v, [3, 1]);
/// assert_eq!(r.to_array(), [13, 11]);
/// ```
///
/// With two input SIMD vectors, the const array specifies element indices in the concatenation of
/// those vectors:
/// ```
/// # #![feature(portable_simd)]
/// # #[cfg(feature = "as_crate")] use core_simd::simd;
/// # #[cfg(not(feature = "as_crate"))] use core::simd;
/// # use simd::{u32x2, u32x4, simd_swizzle};
/// let a = u32x4::from_array([0, 1, 2, 3]);
/// let b = u32x4::from_array([4, 5, 6, 7]);
///
/// // Keeping the same size
/// let r: u32x4 = simd_swizzle!(a, b, [0, 1, 6, 7]);
/// assert_eq!(r.to_array(), [0, 1, 6, 7]);
///
/// // Changing the number of elements
/// let r: u32x2 = simd_swizzle!(a, b, [0, 4]);
/// assert_eq!(r.to_array(), [0, 4]);
/// ```
#[allow(unused_macros)]
pub macro simd_swizzle {
    (
        $vector:expr, $index:expr $(,)?
    ) => {
        {
            use $crate::simd::Swizzle;
            struct Impl;
            impl Swizzle<{$index.len()}> for Impl {
                const INDEX: [usize; {$index.len()}] = $index;
            }
            Impl::swizzle($vector)
        }
    },
    (
        $first:expr, $second:expr, $index:expr $(,)?
    ) => {
        {
            use $crate::simd::Swizzle;
            struct Impl;
            impl Swizzle<{$index.len()}> for Impl {
                const INDEX: [usize; {$index.len()}] = $index;
            }
            Impl::concat_swizzle($first, $second)
        }
    }
}

/// Create a vector from the elements of another vector.
pub trait Swizzle<const N: usize> {
    /// Map from the elements of the input vector to the output vector.
    const INDEX: [usize; N];

    /// Create a new vector from the elements of `vector`.
    ///
    /// Lane `i` of the output is `vector[Self::INDEX[i]]`.
    #[inline]
    #[must_use = "method returns a new vector and does not mutate the original inputs"]
    fn swizzle<T, const M: usize>(vector: Simd<T, M>) -> Simd<T, N>
    where
        T: SimdElement,
        LaneCount<N>: SupportedLaneCount,
        LaneCount<M>: SupportedLaneCount,
    {
        // Safety: `vector` is a vector, and the index is a const array of u32.
        unsafe {
            intrinsics::simd_shuffle(
                vector,
                vector,
                const {
                    let mut output = [0; N];
                    let mut i = 0;
                    while i < N {
                        let index = Self::INDEX[i];
                        assert!(index as u32 as usize == index);
                        assert!(
                            index < M,
                            "source element index exceeds input vector length"
                        );
                        output[i] = index as u32;
                        i += 1;
                    }
                    output
                },
            )
        }
    }

    /// Create a new vector from the elements of `first` and `second`.
    ///
    /// Lane `i` of the output is `concat[Self::INDEX[i]]`, where `concat` is the concatenation of
    /// `first` and `second`.
    #[inline]
    #[must_use = "method returns a new vector and does not mutate the original inputs"]
    fn concat_swizzle<T, const M: usize>(first: Simd<T, M>, second: Simd<T, M>) -> Simd<T, N>
    where
        T: SimdElement,
        LaneCount<N>: SupportedLaneCount,
        LaneCount<M>: SupportedLaneCount,
    {
        // Safety: `first` and `second` are vectors, and the index is a const array of u32.
        unsafe {
            intrinsics::simd_shuffle(
                first,
                second,
                const {
                    let mut output = [0; N];
                    let mut i = 0;
                    while i < N {
                        let index = Self::INDEX[i];
                        assert!(index as u32 as usize == index);
                        assert!(
                            index < 2 * M,
                            "source element index exceeds input vector length"
                        );
                        output[i] = index as u32;
                        i += 1;
                    }
                    output
                },
            )
        }
    }

    /// Create a new mask from the elements of `mask`.
    ///
    /// Element `i` of the output is `concat[Self::INDEX[i]]`, where `concat` is the concatenation of
    /// `first` and `second`.
    #[inline]
    #[must_use = "method returns a new mask and does not mutate the original inputs"]
    fn swizzle_mask<T, const M: usize>(mask: Mask<T, M>) -> Mask<T, N>
    where
        T: MaskElement,
        LaneCount<N>: SupportedLaneCount,
        LaneCount<M>: SupportedLaneCount,
    {
        // SAFETY: all elements of this mask come from another mask
        unsafe { Mask::from_int_unchecked(Self::swizzle(mask.to_int())) }
    }

    /// Create a new mask from the elements of `first` and `second`.
    ///
    /// Element `i` of the output is `concat[Self::INDEX[i]]`, where `concat` is the concatenation of
    /// `first` and `second`.
    #[inline]
    #[must_use = "method returns a new mask and does not mutate the original inputs"]
    fn concat_swizzle_mask<T, const M: usize>(first: Mask<T, M>, second: Mask<T, M>) -> Mask<T, N>
    where
        T: MaskElement,
        LaneCount<N>: SupportedLaneCount,
        LaneCount<M>: SupportedLaneCount,
    {
        // SAFETY: all elements of this mask come from another mask
        unsafe { Mask::from_int_unchecked(Self::concat_swizzle(first.to_int(), second.to_int())) }
    }
}

impl<T, const LANES: usize> Simd<T, LANES>
where
    T: SimdElement,
    LaneCount<LANES>: SupportedLaneCount,
{
    /// Reverse the order of the elements in the vector.
    #[inline]
    #[must_use = "method returns a new vector and does not mutate the original inputs"]
    pub fn reverse(self) -> Self {
        struct Reverse;

        impl<const N: usize> Swizzle<N> for Reverse {
            const INDEX: [usize; N] = const {
                let mut index = [0; N];
                let mut i = 0;
                while i < N {
                    index[i] = N - i - 1;
                    i += 1;
                }
                index
            };
        }

        Reverse::swizzle(self)
    }

    /// Rotates the vector such that the first `OFFSET` elements of the slice move to the end
    /// while the last `LANES - OFFSET` elements move to the front. After calling `rotate_lanes_left`,
    /// the element previously in lane `OFFSET` will become the first element in the slice.
    #[inline]
    #[must_use = "method returns a new vector and does not mutate the original inputs"]
    pub fn rotate_lanes_left<const OFFSET: usize>(self) -> Self {
        struct Rotate<const OFFSET: usize>;

        impl<const OFFSET: usize, const N: usize> Swizzle<N> for Rotate<OFFSET> {
            const INDEX: [usize; N] = const {
                let offset = OFFSET % N;
                let mut index = [0; N];
                let mut i = 0;
                while i < N {
                    index[i] = (i + offset) % N;
                    i += 1;
                }
                index
            };
        }

        Rotate::<OFFSET>::swizzle(self)
    }

    /// Rotates the vector such that the first `LANES - OFFSET` elements of the vector move to
    /// the end while the last `OFFSET` elements move to the front. After calling `rotate_lanes_right`,
    /// the element previously at index `LANES - OFFSET` will become the first element in the slice.
    #[inline]
    #[must_use = "method returns a new vector and does not mutate the original inputs"]
    pub fn rotate_lanes_right<const OFFSET: usize>(self) -> Self {
        struct Rotate<const OFFSET: usize>;

        impl<const OFFSET: usize, const N: usize> Swizzle<N> for Rotate<OFFSET> {
            const INDEX: [usize; N] = const {
                let offset = N - OFFSET % N;
                let mut index = [0; N];
                let mut i = 0;
                while i < N {
                    index[i] = (i + offset) % N;
                    i += 1;
                }
                index
            };
        }

        Rotate::<OFFSET>::swizzle(self)
    }

    /// Interleave two vectors.
    ///
    /// The resulting vectors contain elements taken alternatively from `self` and `other`, first
    /// filling the first result, and then the second.
    ///
    /// The reverse of this operation is [`Simd::deinterleave`].
    ///
    /// ```
    /// # #![feature(portable_simd)]
    /// # use core::simd::Simd;
    /// let a = Simd::from_array([0, 1, 2, 3]);
    /// let b = Simd::from_array([4, 5, 6, 7]);
    /// let (x, y) = a.interleave(b);
    /// assert_eq!(x.to_array(), [0, 4, 1, 5]);
    /// assert_eq!(y.to_array(), [2, 6, 3, 7]);
    /// ```
    #[inline]
    #[must_use = "method returns a new vector and does not mutate the original inputs"]
    pub fn interleave(self, other: Self) -> (Self, Self) {
        const fn interleave<const N: usize>(high: bool) -> [usize; N] {
            let mut idx = [0; N];
            let mut i = 0;
            while i < N {
                let dst_index = if high { i + N } else { i };
                let src_index = dst_index / 2 + (dst_index % 2) * N;
                idx[i] = src_index;
                i += 1;
            }
            idx
        }

        struct Lo;
        struct Hi;

        impl<const N: usize> Swizzle<N> for Lo {
            const INDEX: [usize; N] = interleave::<N>(false);
        }

        impl<const N: usize> Swizzle<N> for Hi {
            const INDEX: [usize; N] = interleave::<N>(true);
        }

        (
            Lo::concat_swizzle(self, other),
            Hi::concat_swizzle(self, other),
        )
    }

    /// Deinterleave two vectors.
    ///
    /// The first result takes every other element of `self` and then `other`, starting with
    /// the first element.
    ///
    /// The second result takes every other element of `self` and then `other`, starting with
    /// the second element.
    ///
    /// The reverse of this operation is [`Simd::interleave`].
    ///
    /// ```
    /// # #![feature(portable_simd)]
    /// # use core::simd::Simd;
    /// let a = Simd::from_array([0, 4, 1, 5]);
    /// let b = Simd::from_array([2, 6, 3, 7]);
    /// let (x, y) = a.deinterleave(b);
    /// assert_eq!(x.to_array(), [0, 1, 2, 3]);
    /// assert_eq!(y.to_array(), [4, 5, 6, 7]);
    /// ```
    #[inline]
    #[must_use = "method returns a new vector and does not mutate the original inputs"]
    pub fn deinterleave(self, other: Self) -> (Self, Self) {
        const fn deinterleave<const N: usize>(second: bool) -> [usize; N] {
            let mut idx = [0; N];
            let mut i = 0;
            while i < N {
                idx[i] = i * 2 + second as usize;
                i += 1;
            }
            idx
        }

        struct Even;
        struct Odd;

        impl<const N: usize> Swizzle<N> for Even {
            const INDEX: [usize; N] = deinterleave::<N>(false);
        }

        impl<const N: usize> Swizzle<N> for Odd {
            const INDEX: [usize; N] = deinterleave::<N>(true);
        }

        (
            Even::concat_swizzle(self, other),
            Odd::concat_swizzle(self, other),
        )
    }
}
