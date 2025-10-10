pub struct FixedBag<T, const N: usize> {
    items: [Option<T>; N],
}

impl<T, const N: usize> FixedBag<T, N> {
    pub const fn new() -> Self {
        FixedBag {
            items: [const { None }; N],
        }
    }

    pub fn push(&mut self, item: T) -> Result<usize, T> {
        let mut idx = 0;
        for slot in &mut self.items {
            if slot.is_none() {
                *slot = Some(item);
                return Ok(idx);
            }
            idx += 1;
        }
        Err(item)
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

    pub fn iter_mut_opt(&mut self) -> impl Iterator<Item = (usize, &mut Option<T>)> {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(i, opt)| if opt.is_some() { Some((i, opt)) } else { None })
    }

    pub fn iter_filter(&mut self, mut keep: impl FnMut(&mut T) -> bool) {
        for i in 0..N {
            if let Some(item) = &mut self.items[i] {
                if !keep(item) {
                    self.items[i] = None;
                }
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|opt| opt.as_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index).and_then(|opt| opt.as_mut())
    }
}
