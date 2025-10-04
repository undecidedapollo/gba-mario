#[derive(Clone)]
pub struct FixedQueue<T: Clone, const N: usize> {
    start_idx: usize,
    next_idx: usize,
    items: [Option<T>; N],
}

impl<T: Clone, const N: usize> FixedQueue<T, N> {
    pub const fn new() -> Self {
        FixedQueue {
            items: [const { None }; N],
            start_idx: 0,
            next_idx: 0,
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.items[self.start_idx].take();
        let is_empty = self.start_idx == self.next_idx && item.is_none();
        if !is_empty {
            self.start_idx = self.start_idx + 1;
            if self.start_idx >= N {
                self.start_idx = 0;
            }
        }

        item
    }

    pub fn push_pop(&mut self, item: T) -> Option<T> {
        let mut to_ret = None;
        if self.start_idx == self.next_idx {
            to_ret = self.pop();
        }
        self.items[self.next_idx] = Some(item);
        self.next_idx = self.next_idx + 1;
        if self.next_idx >= N {
            self.next_idx = 0;
        }

        return to_ret;
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx >= N {
            panic!("Index out of bounds on FixedQueue");
        }
        let mut idx_to_get = self.start_idx + idx;
        if idx_to_get >= N {
            idx_to_get -= N;
        }
        self.items[idx_to_get].as_ref()
    }

    pub fn clear(&mut self) {
        for slot in &mut self.items {
            *slot = None;
        }
        self.start_idx = 0;
        self.next_idx = 0;
    }
}
