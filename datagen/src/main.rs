use crate::room::generate_rooms;

pub mod room;

fn main() {
    std::fs::create_dir("./assets/generated").expect("Failed to create directory");
    generate_rooms().expect("Failed to generate rooms");
}
