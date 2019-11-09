pub trait Validate {
	fn is_valid(&self) -> bool;
}

mod model;
mod block_state;
mod font;

pub use model::ItemModel;
pub use block_state::BlockState;
pub use font::Font;