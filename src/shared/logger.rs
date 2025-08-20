use core::fmt;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Once,
};

use env_logger::fmt::{Color, Style, StyledValue};
use log::Level;

use crate::config::CARGO_PKG_NAME;

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

static SETUP_LOGS: Once = Once::new();

#[cfg(test)]
pub fn setup_logger() {
    SETUP_LOGS.call_once(|| {
        build_logger().is_test(true).init();
    });
}

#[cfg(not(test))]
pub fn setup_logger() {
    SETUP_LOGS.call_once(|| {
        build_logger().is_test(false).init();
    });
}

fn build_logger() -> env_logger::Builder {
    let mut builder = env_logger::Builder::new();

    let pkg_name = CARGO_PKG_NAME.clone();
    let pkg_name_len = pkg_name.len();

    builder.format(move |f, record| {
        use std::io::Write;
        let mut target = record.target();
        if target.starts_with(&pkg_name) {
            if target.len() == pkg_name_len {
                target = "db";
            } else {
                target = &target[pkg_name.len() + 2..];
            }
        }

        let max_width = max_target_width(target);

        let mut style = f.style();
        let level = colored_level(&mut style, record.level());

        let mut style = f.style();
        let target = style.set_bold(true).value(Padded {
            value: target,
            width: max_width,
        });

        let time = format!("{t}", t = f.timestamp_micros());
        let time = &time[11..]; // skip date
        writeln!(f, "{time} {level} {target} > {}", record.args(),)
    });

    if std::env::var_os("RUST_LOG").is_none() {
        builder.filter_level(log::LevelFilter::Info);
    }

    builder.parse_env("RUST_LOG");

    builder
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
