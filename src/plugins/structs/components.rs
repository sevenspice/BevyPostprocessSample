/*
  ポストプロセス設定構造体
  ※ こちらに修正を加えたらシェーダー側に定義している構造体も同じように修正を加えること
*/
use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::*,
    },
};
use crate::consts::app::*;

#[derive(Clone, Copy, ShaderType)]
pub struct DitherSettings {
    pub is_enable:        u32 // ディザを適用するかどうか 1=ON 0=OFF
    , pub is_monochrome:  u32 // モノクロディザにするかどうか 1=ON 0=OFF
    , pub intensity:      f32 // グレースケール閾値
    , pub scale:          i32 // ディザのスケール
    , pub weight_scaling: f32 // 閾値ごとにかけるディザ（ベイヤー行列）を決めるための係数、数値が大きいほどグレーの濃淡の識別が増えるがベイヤー行列の種類数に合わせないと意味がないので注意
    , pub _pad_0:         f32
    , pub _pad_1:         f32
    , pub _pad_2:         f32
}

impl Default for DitherSettings {
    fn default() -> Self {
        Self {
            is_enable:        DEFAULT_DITHER_ENABLE
            , is_monochrome:  DEFAULT_DITHER_MONOCHROME
            , intensity:      DEFAULT_DITHER_INTENSITY
            , scale:          DEFAULT_DITHER_SCALE
            , weight_scaling: DEFAULT_WEIGHT_SCALE
            , _pad_0: 0.0
            , _pad_1: 0.0
            , _pad_2: 0.0
        }
    }
}

#[derive(Clone, Copy, ShaderType)]
pub struct EdgeSettings {
    pub is_enable:       u32  // エッジを適用するかどうか 1=ON 0=OFF
    , pub edge_strength: f32  // エッジ強度の検出閾値
    ,
    #[cfg(feature = "webgl2")]
    pub _edge_padding: Vec2
}

impl Default for EdgeSettings {
    fn default() -> Self {
        Self {
            is_enable:       DEFAULT_EDGE_ENABLE
            , edge_strength: DEFAULT_EDGE_STRENGTH
            ,
            #[cfg(feature = "webgl2")]
            _edge_padding: Vec2::ZERO
        }
    }
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct PostProcessSettings {
    pub is_enable:       u32 // ポストプロセスを適用するかどうか 1=ON 0=OFF
    , pub screen_width:  f32 // 描画幅
    , pub screen_height: f32 // 描画高さ
    , _pad_0:            f32
    , dither: DitherSettings
    , edge: EdgeSettings
    ,
    #[cfg(feature = "webgl2")]
    pub _webgl2_padding: Vec3,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            is_enable: DEFAULT_POSTPROCESS_ENABLE
            , screen_width: GAME_WIDTH
            , screen_height: GAME_HEIGHT
            , _pad_0: 0.0
            , dither: DitherSettings::default()
            , edge: EdgeSettings::default()
            ,
            #[cfg(feature = "webgl2")]
            _webgl2_padding: Vec3::ZERO,
        }
    }
}
