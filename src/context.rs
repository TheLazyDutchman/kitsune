pub struct Context<T> {
	stage: T,
}

impl<T> Context<T> {
	pub fn new(stage: T) -> Self {
		Self { stage }
	}
}

impl<T> std::ops::Deref for Context<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.stage
	}
}

impl<T> std::ops::DerefMut for Context<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.stage
	}
}
