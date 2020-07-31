pub trait UserInputs {
	fn is_mouse_down(&self) -> bool;
	fn mouse_position(&self) -> (i32, i32);
	fn wheel_position(&self) -> (f32, f32);
}
