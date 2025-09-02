use bevy::prelude::*;

// 全般
pub const CLEAR_WINDOW_COLOR: Color = Color::srgb(0.04, 0.04, 0.04);
pub const GAME_WIDTH:  f32 = 1280.;
pub const GAME_HEIGHT: f32 = 720.;

// UI
pub const ASSETS_FONT_PATH: &str = "fonts/MoralerspaceXenon/MoralerspaceXenon-Regular.ttf";

// シェーダーポストプロセス
pub const DEFAULT_SHADER_PATH: &str       = "shaders/post_process.wgsl";
pub const DEFAULT_POSTPROCESS_ENABLE: u32 = 1;    // ポストプロセスを適用するかどうか 1=ON 0=OFF
pub const DEFAULT_DITHER_ENABLE: u32      = 1;    // ディザを適用するかどうか 1=ON 0=OFF
pub const DEFAULT_DITHER_MONOCHROME: u32  = 0;    // モノクロディザにするかどうか 1=ON 0=OFF
pub const DEFAULT_DITHER_INTENSITY: f32   = 0.01; // ディザをかけるグレースケールの色式値
pub const DEFAULT_DITHER_SCALE: i32       = 2;    // ディザのスケール
pub const DEFAULT_WEIGHT_SCALE: f32       = 2.0;  // 閾値ごとにかけるディザ（ベイヤー行列）を決めるための係数、数値が大きいほどグレーの濃淡の識別が増えるがベイヤー行列の種類数に合わせないと意味がないので注意
pub const DEFAULT_EDGE_ENABLE: u32        = 1;    // エッジを適用するかどうか 1=ON 0=OFF
pub const DEFAULT_EDGE_STRENGTH: f32      = 0.05; // エッジ強度の検出式値
