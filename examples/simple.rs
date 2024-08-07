use std::fmt::Debug;

use arc_wrapper::arc_wrapper;

#[arc_wrapper(
    vis = "pub",
    rwlock(
        read(method = "read_guard", vis = "hidden"),
        write(method = "write_guard", vis = "pub(crate)")
    )
)]
struct Rw {}

#[derive(Debug)]
#[arc_wrapper(
    derive(Clone, Debug),
    mutex(method = "mutex_guard", doc = r"return the MutexGuard")
)]
pub struct WithGenerics<T: Clone, U: Debug>
where
    T: Debug,
    U: Clone,
{
    _a: (T, U),
}

#[arc_wrapper(lock = "mutex")]
pub struct OrderLockMutex {}

#[arc_wrapper(lock = "rwlock")]
pub struct OrderLockRwlock {}

#[arc_wrapper(lock = "none")]
pub struct OrderLockNolock {}

mod inner {
    #[arc_wrapper::arc_wrapper(mutex)]
    pub struct Export {
        pub _a: i32,
    }

    #[arc_wrapper::arc_wrapper(mutex, vis = "hidden")]
    pub struct NotExport {
        pub _a: i32,
    }
}

fn main() {
    let a = Rw {};
    let a = ArcRw::from(a);
    drop(a.read_guard());
    drop(a.write_guard());
    // let _a2 = a.clone();

    let wg = WithGenerics {
        _a: ("abccde", 114514),
    };
    let wg = ArcWithGenerics::from(wg);
    drop(wg.mutex_guard());
    let _wg2 = wg.clone();

    _ = inner::ArcExport::from(inner::Export { _a: 0 });
    // _ = inner::ArcNotExport::from(inner::NotExport { _a: 0 });
}
