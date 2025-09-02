#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct DitherSettings {
    is_enable:        u32
    , is_monochrome:  u32
    , intensity:      f32
    , scale:          i32
    , weight_scaling: f32
    , _pad_0:         f32
    , _pad_1:         f32
    , _pad_2:         f32
}

struct EdgeSettings {
    is_enable:       u32 // エッジを適用するかどうか 1=ON 0=OFF
    , edge_strength: f32 // エッジ強度の検出閾値
#ifdef SIXTEEN_BYTE_ALIGNMENT
    , _edge_padding: vec2<u32>
#endif
}

struct PostProcessSettings {
    is_enable:       u32 // ポストプロセスを適用するかどうか
    , screen_width:  f32 // 描画幅
    , screen_height: f32 // 描画高さ
    , _pad_0:        f32 // ビット埋め（使わない）
    , dither: DitherSettings
    , edge: EdgeSettings
#ifdef SIXTEEN_BYTE_ALIGNMENT
    , _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

// -------------------- PICO-8 palette & mapping --------------------
const PICO8_COLORS: array<vec3<f32>, 16> = array<vec3<f32>, 16>(
    vec3(0.000, 0.000, 0.000),  //  0 black
    vec3(0.114, 0.169, 0.325),  //  1 dark-blue   (29,43,83)
    vec3(0.494, 0.145, 0.325),  //  2 dark-purple (126,37,83)
    vec3(0.000, 0.529, 0.318),  //  3 dark-green  (0,135,81)
    vec3(0.671, 0.322, 0.212),  //  4 brown       (171,82,54)
    vec3(0.373, 0.341, 0.310),  //  5 dark-gray   (95,87,79)
    vec3(0.761, 0.765, 0.780),  //  6 light-gray  (194,195,199)
    vec3(1.000, 0.945, 0.910),  //  7 white       (255,241,232)
    vec3(1.000, 0.000, 0.302),  //  8 red         (255,0,77)
    vec3(1.000, 0.639, 0.000),  //  9 orange      (255,163,0)
    vec3(1.000, 0.925, 0.153),  // 10 yellow      (255,236,39)
    vec3(0.000, 0.894, 0.212),  // 11 green       (0,228,54)
    vec3(0.161, 0.678, 1.000),  // 12 blue        (41,173,255)
    vec3(0.514, 0.463, 0.612),  // 13 lilac       (131,118,156)
    vec3(1.000, 0.467, 0.659),  // 14 pink        (255,119,168)
    vec3(1.000, 0.800, 0.667)   // 15 peach       (255,204,170)
);

//
// === Bayer 2x2 ===
//
fn bayer2x2(x: i32, y: i32) -> f32 {
    let mat = array<i32, 4>(0, 2, 3, 1);
    let tx = x % 2;
    let ty = y % 2;
    return f32(mat[ty * 2 + tx]) / 4.0;
}

//
// === Bayer 4x4 ===
//
fn bayer4x4(x: i32, y: i32) -> f32 {
    let mat = array<i32, 16>(
         0,  8,  2, 10,
        12,  4, 14,  6,
         3, 11,  1,  9,
        15,  7, 13,  5
    );
    let tx = x % 4;
    let ty = y % 4;
    return f32(mat[ty * 4 + tx]) / 16.0;
}

//
// === Bayer 8x8 ===
//
fn bayer8x8(x: i32, y: i32) -> f32 {
    let mat = array<i32, 64>(
         0, 32,  8, 40,  2, 34, 10, 42,
        48, 16, 56, 24, 50, 18, 58, 26,
        12, 44,  4, 36, 14, 46,  6, 38,
        60, 28, 52, 20, 62, 30, 54, 22,
         3, 35, 11, 43,  1, 33,  9, 41,
        51, 19, 59, 27, 49, 17, 57, 25,
        15, 47,  7, 39, 13, 45,  5, 37,
        63, 31, 55, 23, 61, 29, 53, 21
    );
    let tx = x % 8;
    let ty = y % 8;
    return f32(mat[ty * 8 + tx]) / 64.0;
}

// ベイヤー行列ID
const MAT_2X2: u32 = 0u;
const MAT_4X4: u32 = 1u;
const MAT_8X8: u32 = 2u;

// PICO-8色ごとに使うベイヤー行列の対応表
const MAT_OF: array<u32, 16> = array<u32, 16>(
    MAT_2X2, //  0 black
    MAT_2X2, //  1 dark-blue
    MAT_2X2, //  2 dark-purple
    MAT_2X2, //  3 dark-green
    MAT_4X4, //  4 brown
    MAT_2X2, //  5 dark-gray
    MAT_4X4, //  6 light-gray
    MAT_8X8, //  7 white
    MAT_4X4, //  8 red
    MAT_8X8, //  9 orange
    MAT_8X8, // 10 yellow
    MAT_4X4, // 11 green
    MAT_8X8, // 12 blue
    MAT_4X4, // 13 lilac
    MAT_8X8, // 14 pink
    MAT_8X8  // 15 peach
);

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // 無効時は何もせず元色を返す
    if settings.is_enable == 0u {
        return textureSample(screen_texture, texture_sampler, in.uv);
    }

    // スクリーンに描画されているテクスチャ（描画イメージ）取得
    let tex_color  = textureSample(screen_texture, texture_sampler, in.uv);
    let base_color = tex_color.rgb;          // ピクセルのオリジナル色
    let comp_color = vec3(1.0) - base_color; // ピクセルのオリジナル色の補色（例: 白↔黒, マゼンタ↔緑）

    // テクスチャをグレースケールに変換
    // ITU-R Rec BT.601
    let gray = dot(tex_color.rgb, vec3(0.299, 0.587, 0.114));
    let near_black = gray <= 0.01; // 黒判定
    let near_white = gray >= 0.99; // 白判定

    // 描画領域のサイズ
    let screen_size = vec2<f32>(settings.screen_width, settings.screen_height);

    // 描画領域の
    let scale = settings.dither.scale;
    let coord = vec2<i32>(
        i32(in.uv.x * screen_size.x) / scale,
        i32(in.uv.y * screen_size.y) / scale
    );

    // ディザの行列
    let t2 = bayer2x2(coord.x, coord.y);
    let t4 = bayer4x4(coord.x, coord.y);
    let t8 = bayer8x8(coord.x, coord.y);

    // グレーの色を0.0～1.0に正規化
    let normalized_gray = clamp((gray - 0.1) / 0.9, 0.0, 1.0);

    //
    // 重み（低輝度→粗い, 高輝度→細かい）
    // 
    // 正規化されたグレーの色数値からどのベイヤー行列を適用するかを決める
    //
    let w2 = clamp(1.0 - normalized_gray * settings.dither.weight_scaling, 0.0, 1.0);
    let w4 = clamp(1.0 - abs(normalized_gray * settings.dither.weight_scaling - 1.5), 0.0, 1.0);
    let w8 = clamp(normalized_gray * settings.dither.weight_scaling - 2.0, 0.0, 1.0);

    // 重みからかけるディザ行列の判断材料となる閾値を算出
    let sum = max(w2 + w4 + w8, 1e-5);
    let threshold = (t2 * w2 + t4 * w4 + t8 * w8) / sum; // ベイヤー行列をかけて重みを計算している

    // エッジ検出 (ピクセルの色値から明暗の差を算出している)
    let offset = vec2<f32>(1.0 / screen_size.x, 1.0 / screen_size.y);
    let left   = dot(textureSample(screen_texture, texture_sampler, in.uv - vec2(offset.x, 0.0)).rgb, vec3(0.299, 0.587, 0.114));
    let right  = dot(textureSample(screen_texture, texture_sampler, in.uv + vec2(offset.x, 0.0)).rgb, vec3(0.299, 0.587, 0.114));
    let top    = dot(textureSample(screen_texture, texture_sampler, in.uv - vec2(0.0, offset.y)).rgb, vec3(0.299, 0.587, 0.114));
    let bottom = dot(textureSample(screen_texture, texture_sampler, in.uv + vec2(0.0, offset.y)).rgb, vec3(0.299, 0.587, 0.114));
    let dx = right - left;
    let dy = bottom - top;
    let edge_strength = length(vec2(dx, dy));
    let is_edge = edge_strength > settings.edge.edge_strength;

    if settings.dither.is_monochrome == 1u {
        var black_or_white: f32;
        if is_edge {
            // エッジと検出されたピクセルは常に白色にする
            black_or_white = 1.0;
        } else if gray < settings.dither.intensity {
            // ピクセルの数値が閾値より小さい場合は常に黒色にする
            black_or_white = 0.0;
        } else if gray > threshold {
            // 閾値を超えたピクセルは白色で返す
            black_or_white = 1.0;
        } else {
            black_or_white = 0.0;
        }

        return vec4(vec3(black_or_white), 1.0);
    } else {
        var out_rgb: vec3<f32>;
        if near_black || near_white {
            // 白と黒はディザなし
            out_rgb = base_color;
        } else if is_edge {
            out_rgb = vec3<f32>(1.0, 1.0, 1.0);
        } else if gray < settings.dither.intensity {
            // 低輝度はそのまま原色で出力
            out_rgb = base_color;
        } else {
            // 元色が黒寄りか白寄りかで、相手を黒/白に自動選択
            let endpoint_is_white = normalized_gray >= 0.5;
            let endpoint = select(vec3(0.0), vec3(1.0), endpoint_is_white);

            // Ordered dither:
            // - 白を相手にする時は g > threshold で相手色（白）を増やす
            // - 黒を相手にする時は g < threshold で相手色（黒）を増やす
            let use_endpoint =
                (endpoint_is_white && (normalized_gray > threshold)) ||
                (!endpoint_is_white && (normalized_gray < threshold));

            out_rgb = select(base_color, endpoint, use_endpoint);
        }
        return vec4(out_rgb, tex_color.a);
    }
}
