#[derive(Clone)]
pub struct FixedBag<T: Clone, const N: usize> {
    items: [Option<T>; N],
}

impl<T: Clone, const N: usize> FixedBag<T, N> {
    pub const fn new() -> Self {
        FixedBag {
            items: [const { None }; N],
        }
    }

    pub fn push(&mut self, item: T) -> usize {
        let mut idx = 0;
        for slot in &mut self.items {
            if slot.is_none() {
                *slot = Some(item);
                return idx;
            }
            idx += 1;
        }
        panic!("FixedBag is full");
    }

    pub fn remove(&mut self, index: usize) {
        if index < N {
            self.items[index] = None;
        }
    }

    pub fn take(&mut self, index: usize) -> Option<T> {
        if index < N {
            self.items[index].take()
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.items {
            *slot = None;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|t| (i, t)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_mut().map(|t| (i, t)))
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|opt| opt.as_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index).and_then(|opt| opt.as_mut())
    }
}
