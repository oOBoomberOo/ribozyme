use crate::rp::Resource;

#[derive(Debug, PartialEq, Eq)]
pub struct Conflict {
	pub conflicts: Vec<Resource>
}

impl Conflict {
	pub fn new(conflicts: Vec<Resource>) -> Conflict {
		Conflict { conflicts }
	}

	pub fn with_resource(resource: Resource) -> Conflict {
		Conflict::new(vec![resource])
	}

	pub fn add(&mut self, resource: Resource) {
		self.conflicts.push(resource);
	}
}