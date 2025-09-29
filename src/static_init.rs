use gba::prelude::GbaCell;

/// Marker trait for types that are safe to initialize as statics in EWRAM.
///
/// # Safety
/// Implementing types must:
/// - Be fully const-constructible (have a `const fn new()` or equivalent)
/// - Not require runtime initialization beyond what can be done in `init()`
/// - Be safe to access from a single mutable reference for the program's lifetime
pub unsafe trait StaticInitSafe {
    /// Called once to perform any runtime initialization needed.
    /// Default implementation does nothing.
    fn init(&mut self) {
        // Default: no-op for types that don't need runtime init
    }
}

/// Wrapper for safely managing single-init access to static data.
pub struct StaticCell<T: StaticInitSafe> {
    has_init: GbaCell<bool>,
    ptr: *mut T,
}

impl<T: StaticInitSafe> StaticCell<T> {
    pub const fn new(ptr: *mut T) -> Self {
        StaticCell {
            ptr,
            has_init: GbaCell::new(false),
        }
    }

    #[track_caller]
    pub fn init(&self) -> &'static mut T {
        if self.has_init.read() {
            panic!("Multiple inits of static");
        }
        self.has_init.write(true);

        let val = unsafe { &mut *self.ptr };
        val.init(); // Call the trait's init method
        val
    }

    /// Get a reference if already initialized, panics otherwise
    #[track_caller]
    pub fn get(&self) -> &'static mut T {
        if !self.has_init.read() {
            panic!("Accessing uninitialized static");
        }
        unsafe { &mut *self.ptr }
    }

    pub fn get_or_init(&self) -> &'static mut T {
        if self.is_init() {
            self.get()
        } else {
            self.init()
        }
    }

    /// Check if initialized without panicking
    pub fn is_init(&self) -> bool {
        self.has_init.read()
    }
}

unsafe impl<T: StaticInitSafe> Sync for StaticCell<T> {}

#[macro_export]
macro_rules! ewram_static {
    ($vis:vis $name:ident: $ty:ty = $init:expr) => {
        #[allow(non_upper_case_globals)]
        $vis static $name: $crate::static_init::StaticCell<$ty> = {
            #[unsafe(link_section = ".ewram")]
            static mut STORAGE: $ty = $init;

            $crate::static_init::StaticCell::new( core::ptr::addr_of_mut!(STORAGE) )
        };
    };
}
