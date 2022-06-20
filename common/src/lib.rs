pub mod prelude;
pub mod utils;

#[macro_export]
macro_rules! operator {
    () => {
        concat!("dev.withlazers.operator.", env!("CARGO_PKG_NAME"))
    };
}

#[macro_export]
macro_rules! label {
    ($name:literal) => {
        concat!(common::operator!(), "/", $name)
    };
    () => {
        label!("")
    };
}

#[macro_export]
macro_rules! annotation {
    ($name:literal) => {
        concat!(common::operator!(), "/", $name)
    };
    () => {
        annotation!("")
    };
}
