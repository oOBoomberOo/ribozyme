use super::Namespace;

use std::convert::TryFrom;
use std::fmt;
use serde::de;
use serde::Deserializer;
use serde::{Deserialize, Serialize};

impl Serialize for Namespace {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let value = self.to_string();
		serializer.serialize_str(&value)
	}
}

impl<'de> Deserialize<'de> for Namespace {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(NamespaceVisitor)
	}
}

struct NamespaceVisitor;

impl<'de> de::Visitor<'de> for NamespaceVisitor {
	type Value = Namespace;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a namespace string")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Namespace::try_from(value).map_err(E::custom)
	}
}
