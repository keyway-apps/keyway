#[macro_export]
macro_rules! internal_modules {
    ($($name:ident $(as $alias:ident)?),+ $(,)?) => {
        $(
            extern crate $name $(as $alias)?;
        )+
    };
}
