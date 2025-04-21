mod config;
mod macros;

// For submodules I only expose the public API to the parent module

pub mod storage {
    mod page;
    mod tuple;

    pub mod disk {
        pub mod manager;
        pub mod scheduler;
    }

    pub use disk::manager::DiskManager;
    pub use page::{PageId, SlottedPage};
}

pub mod catalog {
    mod schema;
}

pub mod buffer_pool {
    mod frame;
    mod lruk;
    mod pool;
    pub use pool::BufferPool;
}

pub mod dbms {
    mod database;
    pub use database::Database;
}
