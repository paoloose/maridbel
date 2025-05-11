mod config;
mod errors;
mod macros;

// For submodules I only expose the public API to the parent module

pub mod storage {
    mod page;
    mod tuple;

    pub mod disk {
        pub mod disk_manager;
        pub mod disk_scheduler;
    }

    pub mod buffer {
        pub mod buffer_pool;
        mod eviction;
        pub mod frame;
        mod lruk_eviction;
    }

    pub use buffer::buffer_pool::BufferPool;
    pub use buffer::frame::Frame;
    pub use disk::disk_manager::DiskManager;
    pub use page::{PageId, SlottedPage};
}

pub mod catalog {
    mod schema;
}

pub mod dbms {
    mod database;
    pub use database::Database;
}
