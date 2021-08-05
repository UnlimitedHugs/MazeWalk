// adapted from bevy_render/src/color.rs
use glam::{vec3, Vec3, Vec4};
use colorspace::*;
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
	/// sRGBA color
	Rgba {
		/// Red component. [0.0, 1.0]
		red: f32,
		/// Green component. [0.0, 1.0]
		green: f32,
		/// Blue component. [0.0, 1.0]
		blue: f32,
		/// Alpha component. [0.0, 1.0]
		alpha: f32,
	},
	/// RGBA color in the Linear sRGB colorspace (often colloquially referred to as "linear",
	/// "RGB", or "linear RGB").
	RgbaLinear {
		/// Red component. [0.0, 1.0]
		red: f32,
		/// Green component. [0.0, 1.0]
		green: f32,
		/// Blue component. [0.0, 1.0]
		blue: f32,
		/// Alpha component. [0.0, 1.0]
		alpha: f32,
	},
	/// HSL (hue, saturation, lightness) color with an alpha channel
	Hsla {
		/// Hue component. [0.0, 360.0]
		hue: f32,
		/// Saturation component. [0.0, 1.0]
		saturation: f32,
		/// Lightness component. [0.0, 1.0]
		lightness: f32,
		/// Alpha component. [0.0, 1.0]
		alpha: f32,
	},
}

impl Color {
	pub const ALICE_BLUE: Color = Color::rgb(0.94, 0.97, 1.0);
	pub const ANTIQUE_WHITE: Color = Color::rgb(0.98, 0.92, 0.84);
	pub const AQUAMARINE: Color = Color::rgb(0.49, 1.0, 0.83);
	pub const AZURE: Color = Color::rgb(0.94, 1.0, 1.0);
	pub const BEIGE: Color = Color::rgb(0.96, 0.96, 0.86);
	pub const BISQUE: Color = Color::rgb(1.0, 0.89, 0.77);
	pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
	pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
	pub const CRIMSON: Color = Color::rgb(0.86, 0.08, 0.24);
	pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
	pub const DARK_GRAY: Color = Color::rgb(0.25, 0.25, 0.25);
	pub const DARK_GREEN: Color = Color::rgb(0.0, 0.5, 0.0);
	pub const FUCHSIA: Color = Color::rgb(1.0, 0.0, 1.0);
	pub const GOLD: Color = Color::rgb(1.0, 0.84, 0.0);
	pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
	pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
	pub const INDIGO: Color = Color::rgb(0.29, 0.0, 0.51);
	pub const LIME_GREEN: Color = Color::rgb(0.2, 0.8, 0.2);
	pub const MAROON: Color = Color::rgb(0.5, 0.0, 0.0);
	pub const MIDNIGHT_BLUE: Color = Color::rgb(0.1, 0.1, 0.44);
	pub const NAVY: Color = Color::rgb(0.0, 0.0, 0.5);
	pub const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
	pub const OLIVE: Color = Color::rgb(0.5, 0.5, 0.0);
	pub const ORANGE: Color = Color::rgb(1.0, 0.65, 0.0);
	pub const ORANGE_RED: Color = Color::rgb(1.0, 0.27, 0.0);
	pub const PINK: Color = Color::rgb(1.0, 0.08, 0.58);
	pub const PURPLE: Color = Color::rgb(0.5, 0.0, 0.5);
	pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
	pub const SALMON: Color = Color::rgb(0.98, 0.5, 0.45);
	pub const SEA_GREEN: Color = Color::rgb(0.18, 0.55, 0.34);
	pub const SILVER: Color = Color::rgb(0.75, 0.75, 0.75);
	pub const TEAL: Color = Color::rgb(0.0, 0.5, 0.5);
	pub const TOMATO: Color = Color::rgb(1.0, 0.39, 0.28);
	pub const TURQUOISE: Color = Color::rgb(0.25, 0.88, 0.82);
	pub const VIOLET: Color = Color::rgb(0.93, 0.51, 0.93);
	pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
	pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
	pub const YELLOW_GREEN: Color = Color::rgb(0.6, 0.8, 0.2);

	/// New `Color` from sRGB colorspace.
	pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
		Color::Rgba {
			red: r,
			green: g,
			blue: b,
			alpha: 1.0,
		}
	}

	/// New `Color` from sRGB colorspace.
	pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
		Color::Rgba {
			red: r,
			green: g,
			blue: b,
			alpha: a,
		}
	}

	/// New `Color` from linear RGB colorspace.
	pub const fn rgb_linear(r: f32, g: f32, b: f32) -> Color {
		Color::RgbaLinear {
			red: r,
			green: g,
			blue: b,
			alpha: 1.0,
		}
	}

	/// New `Color` from linear RGB colorspace.
	pub const fn rgba_linear(r: f32, g: f32, b: f32, a: f32) -> Color {
		Color::RgbaLinear {
			red: r,
			green: g,
			blue: b,
			alpha: a,
		}
	}

	/// New `Color` with HSL representation in sRGB colorspace.
	pub const fn hsl(hue: f32, saturation: f32, lightness: f32) -> Color {
		Color::Hsla {
			hue,
			saturation,
			lightness,
			alpha: 1.0,
		}
	}

	/// New `Color` with HSL representation in sRGB colorspace.
	pub const fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Color {
		Color::Hsla {
			hue,
			saturation,
			lightness,
			alpha,
		}
	}

	/// New `Color` from sRGB colorspace.
	pub fn rgb_u8(r: u8, g: u8, b: u8) -> Color {
		Color::rgba_u8(r, g, b, u8::MAX)
	}

	// Float operations in const fn are not stable yet
	// see https://github.com/rust-lang/rust/issues/57241
	/// New `Color` from sRGB colorspace.
	pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
		Color::rgba(
			r as f32 / u8::MAX as f32,
			g as f32 / u8::MAX as f32,
			b as f32 / u8::MAX as f32,
			a as f32 / u8::MAX as f32,
		)
	}

	/// Decode Color from RGB integer
	pub fn rgb_u32(c: u32) -> Color {
		Color::rgb_u8(
			((c >> 16) & 0xFF) as u8,
			((c >> 8) & 0xFF) as u8,
			((c >> 0) & 0xFF) as u8,
		)
	}

	/// Get red in sRGB colorspace.
	pub fn r(&self) -> f32 {
		match self.as_rgba() {
			Color::Rgba { red, .. } => red,
			_ => unreachable!(),
		}
	}

	/// Get green in sRGB colorspace.
	pub fn g(&self) -> f32 {
		match self.as_rgba() {
			Color::Rgba { green, .. } => green,
			_ => unreachable!(),
		}
	}

	/// Get blue in sRGB colorspace.
	pub fn b(&self) -> f32 {
		match self.as_rgba() {
			Color::Rgba { blue, .. } => blue,
			_ => unreachable!(),
		}
	}

	/// Set red in sRGB colorspace.
	pub fn set_r(&mut self, r: f32) -> &mut Self {
		*self = self.as_rgba();
		match self {
			Color::Rgba { red, .. } => *red = r,
			_ => unreachable!(),
		}
		self
	}

	/// Set green in sRGB colorspace.
	pub fn set_g(&mut self, g: f32) -> &mut Self {
		*self = self.as_rgba();
		match self {
			Color::Rgba { green, .. } => *green = g,
			_ => unreachable!(),
		}
		self
	}

	/// Set blue in sRGB colorspace.
	pub fn set_b(&mut self, b: f32) -> &mut Self {
		*self = self.as_rgba();
		match self {
			Color::Rgba { blue, .. } => *blue = b,
			_ => unreachable!(),
		}
		self
	}

	/// Get alpha.
	pub fn a(&self) -> f32 {
		match self {
			Color::Rgba { alpha, .. }
			| Color::RgbaLinear { alpha, .. }
			| Color::Hsla { alpha, .. } => *alpha,
		}
	}

	/// Set alpha.
	pub fn set_a(&mut self, a: f32) -> &mut Self {
		match self {
			Color::Rgba { alpha, .. }
			| Color::RgbaLinear { alpha, .. }
			| Color::Hsla { alpha, .. } => {
				*alpha = a;
			}
		}
		self
	}

	/// Converts a `Color` to variant `Color::Rgba`
	pub fn as_rgba(self: &Color) -> Color {
		match self {
			Color::Rgba { .. } => *self,
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => Color::Rgba {
				red: red.linear_to_nonlinear_srgb(),
				green: green.linear_to_nonlinear_srgb(),
				blue: blue.linear_to_nonlinear_srgb(),
				alpha: *alpha,
			},
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				let [red, green, blue] =
					HslRepresentation::hsl_to_nonlinear_srgb(*hue, *saturation, *lightness);
				Color::Rgba {
					red,
					green,
					blue,
					alpha: *alpha,
				}
			}
		}
	}

	/// Converts a `Color` to variant `Color::RgbaLinear`
	pub fn as_rgba_linear(self: &Color) -> Color {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => Color::Rgba {
				red: red.nonlinear_to_linear_srgb(),
				green: green.nonlinear_to_linear_srgb(),
				blue: blue.nonlinear_to_linear_srgb(),
				alpha: *alpha,
			},
			Color::RgbaLinear { .. } => *self,
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				let [red, green, blue] =
					HslRepresentation::hsl_to_nonlinear_srgb(*hue, *saturation, *lightness);
				Color::RgbaLinear {
					red: red.nonlinear_to_linear_srgb(),
					green: green.nonlinear_to_linear_srgb(),
					blue: blue.nonlinear_to_linear_srgb(),
					alpha: *alpha,
				}
			}
		}
	}

	/// Converts a `Color` to variant `Color::Hsla`
	pub fn as_hsla(self: &Color) -> Color {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => {
				let (hue, saturation, lightness) =
					HslRepresentation::nonlinear_srgb_to_hsl([*red, *green, *blue]);
				Color::Hsla {
					hue,
					saturation,
					lightness,
					alpha: *alpha,
				}
			}
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => {
				let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([
					red.linear_to_nonlinear_srgb(),
					green.linear_to_nonlinear_srgb(),
					blue.linear_to_nonlinear_srgb(),
				]);
				Color::Hsla {
					hue,
					saturation,
					lightness,
					alpha: *alpha,
				}
			}
			Color::Hsla { .. } => *self,
		}
	}

	/// Converts a `Color` to a `[f32; 4]` from sRBG colorspace
	pub fn as_rgba_f32(self: Color) -> [f32; 4] {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => [red, green, blue, alpha],
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => [
				red.linear_to_nonlinear_srgb(),
				green.linear_to_nonlinear_srgb(),
				blue.linear_to_nonlinear_srgb(),
				alpha,
			],
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				let [red, green, blue] =
					HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
				[red, green, blue, alpha]
			}
		}
	}

	/// Converts a `Color` to a `[f32; 4]` from linear RBG colorspace
	pub fn as_linear_rgba_f32(self: Color) -> [f32; 4] {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => [
				red.nonlinear_to_linear_srgb(),
				green.nonlinear_to_linear_srgb(),
				blue.nonlinear_to_linear_srgb(),
				alpha,
			],
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => [red, green, blue, alpha],
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				let [red, green, blue] =
					HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
				[
					red.nonlinear_to_linear_srgb(),
					green.nonlinear_to_linear_srgb(),
					blue.nonlinear_to_linear_srgb(),
					alpha,
				]
			}
		}
	}

	/// Converts a `Color` to a `[f32; 4]` from HLS colorspace
	pub fn as_hlsa_f32(self: Color) -> [f32; 4] {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => {
				let (hue, saturation, lightness) =
					HslRepresentation::nonlinear_srgb_to_hsl([red, green, blue]);
				[hue, saturation, lightness, alpha]
			}
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => {
				let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([
					red.linear_to_nonlinear_srgb(),
					green.linear_to_nonlinear_srgb(),
					blue.linear_to_nonlinear_srgb(),
				]);
				[hue, saturation, lightness, alpha]
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => [hue, saturation, lightness, alpha],
		}
	}
}

impl Default for Color {
	fn default() -> Self {
		Color::WHITE
	}
}

impl AddAssign<Color> for Color {
	fn add_assign(&mut self, rhs: Color) {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => {
				let rhs = rhs.as_rgba_f32();
				*red += rhs[0];
				*green += rhs[1];
				*blue += rhs[2];
				*alpha += rhs[3];
			}
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => {
				let rhs = rhs.as_linear_rgba_f32();
				*red += rhs[0];
				*green += rhs[1];
				*blue += rhs[2];
				*alpha += rhs[3];
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				let rhs = rhs.as_linear_rgba_f32();
				*hue += rhs[0];
				*saturation += rhs[1];
				*lightness += rhs[2];
				*alpha += rhs[3];
			}
		}
	}
}

impl Add<Color> for Color {
	type Output = Color;

	fn add(self, rhs: Color) -> Self::Output {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => {
				let rhs = rhs.as_rgba_f32();
				Color::Rgba {
					red: red + rhs[0],
					green: green + rhs[1],
					blue: blue + rhs[2],
					alpha: alpha + rhs[3],
				}
			}
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => {
				let rhs = rhs.as_linear_rgba_f32();
				Color::RgbaLinear {
					red: red + rhs[0],
					green: green + rhs[1],
					blue: blue + rhs[2],
					alpha: alpha + rhs[3],
				}
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				let rhs = rhs.as_linear_rgba_f32();
				Color::Hsla {
					hue: hue + rhs[0],
					saturation: saturation + rhs[1],
					lightness: lightness + rhs[2],
					alpha: alpha + rhs[3],
				}
			}
		}
	}
}

impl AddAssign<Vec4> for Color {
	fn add_assign(&mut self, rhs: Vec4) {
		let rhs: Color = rhs.into();
		*self += rhs
	}
}

impl Add<Vec4> for Color {
	type Output = Color;

	fn add(self, rhs: Vec4) -> Self::Output {
		let rhs: Color = rhs.into();
		self + rhs
	}
}

impl From<Color> for [f32; 4] {
	fn from(color: Color) -> Self {
		color.as_rgba_f32()
	}
}

impl From<[f32; 4]> for Color {
	fn from([r, g, b, a]: [f32; 4]) -> Self {
		Color::rgba(r, g, b, a)
	}
}

impl From<Color> for Vec4 {
	fn from(color: Color) -> Self {
		let color: [f32; 4] = color.into();
		Vec4::new(color[0], color[1], color[2], color[3])
	}
}

impl From<Vec4> for Color {
	fn from(vec4: Vec4) -> Self {
		Color::rgba(vec4.x, vec4.y, vec4.z, vec4.w)
	}
}

impl Mul<f32> for Color {
	type Output = Color;

	fn mul(self, rhs: f32) -> Self::Output {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => Color::Rgba {
				red: red * rhs,
				green: green * rhs,
				blue: blue * rhs,
				alpha,
			},
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => Color::RgbaLinear {
				red: red * rhs,
				green: green * rhs,
				blue: blue * rhs,
				alpha,
			},
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => Color::Hsla {
				hue: hue * rhs,
				saturation: saturation * rhs,
				lightness: lightness * rhs,
				alpha,
			},
		}
	}
}

impl MulAssign<f32> for Color {
	fn mul_assign(&mut self, rhs: f32) {
		match self {
			Color::Rgba {
				red, green, blue, ..
			} => {
				*red *= rhs;
				*green *= rhs;
				*blue *= rhs;
			}
			Color::RgbaLinear {
				red, green, blue, ..
			} => {
				*red *= rhs;
				*green *= rhs;
				*blue *= rhs;
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				..
			} => {
				*hue *= rhs;
				*saturation *= rhs;
				*lightness *= rhs;
			}
		}
	}
}

impl Mul<Vec4> for Color {
	type Output = Color;

	fn mul(self, rhs: Vec4) -> Self::Output {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => Color::Rgba {
				red: red * rhs.x,
				green: green * rhs.y,
				blue: blue * rhs.z,
				alpha: alpha * rhs.w,
			},
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => Color::RgbaLinear {
				red: red * rhs.x,
				green: green * rhs.y,
				blue: blue * rhs.z,
				alpha: alpha * rhs.w,
			},
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => Color::Hsla {
				hue: hue * rhs.x,
				saturation: saturation * rhs.y,
				lightness: lightness * rhs.z,
				alpha: alpha * rhs.w,
			},
		}
	}
}

impl MulAssign<Vec4> for Color {
	fn mul_assign(&mut self, rhs: Vec4) {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => {
				*red *= rhs.x;
				*green *= rhs.y;
				*blue *= rhs.z;
				*alpha *= rhs.w;
			}
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => {
				*red *= rhs.x;
				*green *= rhs.y;
				*blue *= rhs.z;
				*alpha *= rhs.w;
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				*hue *= rhs.x;
				*saturation *= rhs.y;
				*lightness *= rhs.z;
				*alpha *= rhs.w;
			}
		}
	}
}

impl Mul<Vec3> for Color {
	type Output = Color;

	fn mul(self, rhs: Vec3) -> Self::Output {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => Color::Rgba {
				red: red * rhs.x,
				green: green * rhs.y,
				blue: blue * rhs.z,
				alpha,
			},
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => Color::RgbaLinear {
				red: red * rhs.x,
				green: green * rhs.y,
				blue: blue * rhs.z,
				alpha,
			},
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => Color::Hsla {
				hue: hue * rhs.x,
				saturation: saturation * rhs.y,
				lightness: lightness * rhs.z,
				alpha,
			},
		}
	}
}

impl MulAssign<Vec3> for Color {
	fn mul_assign(&mut self, rhs: Vec3) {
		match self {
			Color::Rgba {
				red, green, blue, ..
			} => {
				*red *= rhs.x;
				*green *= rhs.y;
				*blue *= rhs.z;
			}
			Color::RgbaLinear {
				red, green, blue, ..
			} => {
				*red *= rhs.x;
				*green *= rhs.y;
				*blue *= rhs.z;
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				..
			} => {
				*hue *= rhs.x;
				*saturation *= rhs.y;
				*lightness *= rhs.z;
			}
		}
	}
}

impl Mul<[f32; 4]> for Color {
	type Output = Color;

	fn mul(self, rhs: [f32; 4]) -> Self::Output {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => Color::Rgba {
				red: red * rhs[0],
				green: green * rhs[1],
				blue: blue * rhs[2],
				alpha: alpha * rhs[3],
			},
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => Color::RgbaLinear {
				red: red * rhs[0],
				green: green * rhs[1],
				blue: blue * rhs[2],
				alpha: alpha * rhs[3],
			},
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => Color::Hsla {
				hue: hue * rhs[0],
				saturation: saturation * rhs[1],
				lightness: lightness * rhs[2],
				alpha: alpha * rhs[3],
			},
		}
	}
}

impl MulAssign<[f32; 4]> for Color {
	fn mul_assign(&mut self, rhs: [f32; 4]) {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => {
				*red *= rhs[0];
				*green *= rhs[1];
				*blue *= rhs[2];
				*alpha *= rhs[3];
			}
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => {
				*red *= rhs[0];
				*green *= rhs[1];
				*blue *= rhs[2];
				*alpha *= rhs[3];
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => {
				*hue *= rhs[0];
				*saturation *= rhs[1];
				*lightness *= rhs[2];
				*alpha *= rhs[3];
			}
		}
	}
}

impl Mul<[f32; 3]> for Color {
	type Output = Color;

	fn mul(self, rhs: [f32; 3]) -> Self::Output {
		match self {
			Color::Rgba {
				red,
				green,
				blue,
				alpha,
			} => Color::Rgba {
				red: red * rhs[0],
				green: green * rhs[1],
				blue: blue * rhs[2],
				alpha,
			},
			Color::RgbaLinear {
				red,
				green,
				blue,
				alpha,
			} => Color::RgbaLinear {
				red: red * rhs[0],
				green: green * rhs[1],
				blue: blue * rhs[2],
				alpha,
			},
			Color::Hsla {
				hue,
				saturation,
				lightness,
				alpha,
			} => Color::Hsla {
				hue: hue * rhs[0],
				saturation: saturation * rhs[1],
				lightness: lightness * rhs[2],
				alpha,
			},
		}
	}
}

impl MulAssign<[f32; 3]> for Color {
	fn mul_assign(&mut self, rhs: [f32; 3]) {
		match self {
			Color::Rgba {
				red, green, blue, ..
			} => {
				*red *= rhs[0];
				*green *= rhs[1];
				*blue *= rhs[2];
			}
			Color::RgbaLinear {
				red, green, blue, ..
			} => {
				*red *= rhs[0];
				*green *= rhs[1];
				*blue *= rhs[2];
			}
			Color::Hsla {
				hue,
				saturation,
				lightness,
				..
			} => {
				*hue *= rhs[0];
				*saturation *= rhs[1];
				*lightness *= rhs[2];
			}
		}
	}
}

impl From<Color> for Vec3 {
	fn from(c: Color) -> Self {
		match c {
			Color::Rgba {
				red, green, blue, ..
			} => vec3(red, green, blue),
			Color::RgbaLinear {
				red, green, blue, ..
			} => vec3(red, green, blue),
			c @ Color::Hsla { .. } => c.as_rgba().into(),
		}
	}
}

mod colorspace {
	pub trait SrgbColorSpace {
		fn linear_to_nonlinear_srgb(self) -> Self;
		fn nonlinear_to_linear_srgb(self) -> Self;
	}

	// source: https://entropymine.com/imageworsener/srgbformula/
	impl SrgbColorSpace for f32 {
		fn linear_to_nonlinear_srgb(self) -> f32 {
			if self <= 0.0 {
				return self;
			}

			if self <= 0.0031308 {
				self * 12.92 // linear falloff in dark values
			} else {
				(1.055 * self.powf(1.0 / 2.4)) - 0.055 // gamma curve in other area
			}
		}

		fn nonlinear_to_linear_srgb(self) -> f32 {
			if self <= 0.0 {
				return self;
			}
			if self <= 0.04045 {
				self / 12.92 // linear falloff in dark values
			} else {
				((self + 0.055) / 1.055).powf(2.4) // gamma curve in other area
			}
		}
	}

	pub struct HslRepresentation;
	impl HslRepresentation {
		/// converts a color in HLS space to sRGB space
		pub fn hsl_to_nonlinear_srgb(hue: f32, saturation: f32, lightness: f32) -> [f32; 3] {
			// https://en.wikipedia.org/wiki/HSL_and_HSV#HSL_to_RGB
			let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
			let hue_prime = hue / 60.0;
			let largest_component = chroma * (1.0 - (hue_prime % 2.0 - 1.0).abs());
			let (r_temp, g_temp, b_temp) = if hue_prime < 1.0 {
				(chroma, largest_component, 0.0)
			} else if hue_prime < 2.0 {
				(largest_component, chroma, 0.0)
			} else if hue_prime < 3.0 {
				(0.0, chroma, largest_component)
			} else if hue_prime < 4.0 {
				(0.0, largest_component, chroma)
			} else if hue_prime < 5.0 {
				(largest_component, 0.0, chroma)
			} else {
				(chroma, 0.0, largest_component)
			};
			let lightness_match = lightness - chroma / 2.0;

			[
				r_temp + lightness_match,
				g_temp + lightness_match,
				b_temp + lightness_match,
			]
		}

		/// converts a color in sRGB space to HLS space
		pub fn nonlinear_srgb_to_hsl([red, green, blue]: [f32; 3]) -> (f32, f32, f32) {
			// https://en.wikipedia.org/wiki/HSL_and_HSV#From_RGB
			let x_max = red.max(green.max(blue));
			let x_min = red.min(green.min(blue));
			let chroma = x_max - x_min;
			let lightness = (x_max + x_min) / 2.0;
			let hue = if chroma == 0.0 {
				0.0
			} else if red > green && red > blue {
				60.0 * (green - blue) / chroma
			} else if green > red && green > blue {
				60.0 * (2.0 + (blue - red) / chroma)
			} else {
				60.0 * (4.0 + (red - green) / chroma)
			};
			let hue = if hue < 0.0 { 360.0 + hue } else { hue };
			let saturation = if lightness <= 0.0 || lightness >= 1.0 {
				0.0
			} else {
				(x_max - lightness) / lightness.min(1.0 - lightness)
			};

			(hue, saturation, lightness)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn color_u32() {
		let c = Color::rgb_u32(0xFF8000);
		assert_eq!(c.r(), 1.0);
		assert_eq!(c.g(), 0.5019608);
		assert_eq!(c.b(), 0.0);
	}
}
