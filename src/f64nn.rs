use std::convert::TryFrom;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct f64s(f64);
impl Eq for f64s {}
impl Ord for f64s {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}
impl PartialEq<f64> for f64s {
	fn eq(&self, f: &f64) -> bool {
		&self.0 == f
	}
}
impl PartialOrd<f64> for f64s {
	fn partial_cmp(&self, f: &f64) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(f)
	}
}
impl TryFrom<f64> for f64s {
	type Error = &'static str;
	fn try_from(f: f64) -> Result<Self, Self::Error> {
		if f.is_nan() {
			Err("NaN")
		} else {
			Ok(Self(f))
		}
	}
}
impl f64s {
	pub const INFINITY: Self = f64s(f64::INFINITY);
	pub const ZERO: Self = f64s(0.0);
	pub fn is_infinite(self) -> bool {
		self.0.is_infinite()
	}
}
impl std::ops::Add<Self> for f64s {
	type Output = Self;
	fn add(self, f: Self) -> Self::Output {
		f64s(self.0 + f.0)
	}
}
impl std::iter::Sum for f64s {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.reduce(std::ops::Add::add).unwrap_or(Self(0.0))
	}
}
impl std::ops::Neg for f64s {
	type Output = Self;
	fn neg(self) -> Self {
		Self(-self.0)
	}
}
