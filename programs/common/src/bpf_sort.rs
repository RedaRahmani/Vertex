//! Tiny helper to avoid stack-heavy stable sort on BPF.
//! On-chain we use `sort_unstable` (lower stack); off-chain keep `sort` for stability.

/// Sort a slice using BPF-safe algorithms.
/// Uses unstable sort on all targets to avoid stack overflow from stable sort frames.
#[inline(always)]
pub fn bpf_sort<T: Ord>(s: &mut [T]) {
    s.sort_unstable();
}

/// Sort a slice using a comparator. Uses unstable sort on BPF.
#[inline(always)]
pub fn bpf_sort_by<T, F>(s: &mut [T], compare: F)
where
    F: FnMut(&T, &T) -> core::cmp::Ordering,
{
    s.sort_unstable_by(compare);
}

/// Sort a slice by a derived key. Uses unstable sort on BPF.
#[inline(always)]
pub fn bpf_sort_by_key<T, F, K>(s: &mut [T], mut f: F)
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    s.sort_unstable_by_key(|x| f(x));
}
