#[macro_export]
macro_rules! marker {
    ($marker:ident) => {
        #[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Default)]
        struct $marker;
    };
    (pub $marker:ident) => {
        #[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Default)]
        pub struct $marker;
    };
}

#[macro_export]
macro_rules! singleton_marker {
    ($marker:ident) => {
        #[derive(Resource, Debug, Clone, Copy, Eq, PartialEq, Default)]
        struct $marker;
    };
    (pub $marker:ident) => {
        #[derive(Resource, Debug, Clone, Copy, Eq, PartialEq, Default)]
        pub struct $marker;
    };
}