mod config;
mod macros;

// For submodules I only expose the public API to the parent module

pub mod storage {
    mod page;
    mod tuple;
}

pub mod catalog {
    mod schema;
}

pub mod buffer_pool {
    mod pool;
    pub use pool::BufferPool;
}

pub mod dbms {
    mod database;
    pub use database::Database;
}
