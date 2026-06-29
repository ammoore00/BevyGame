#[macro_export]
macro_rules! define_sprite_resource {
    ($name:ident, $path:literal) => {
        paste::paste! {
            $crate::define_resource!(
                [<$name Sprite>],
                concat!("images/", $path),
                Image,
                ResourceFileType::Image
            );
        }
    };
}