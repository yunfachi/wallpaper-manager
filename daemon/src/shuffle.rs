use std::hash::{BuildHasher, Hasher, RandomState};

pub fn shuffle<T>(vec: &mut [T]) {
    let n = vec.len();
    for i in 0..(n - 1) {
        let j = (RandomState::new().build_hasher().finish() as usize) % (n - i) + i;
        vec.swap(i, j);
    }
}
