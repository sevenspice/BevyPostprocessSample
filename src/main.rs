pub mod consts;
pub mod resources;
pub mod plugins;

// --- Bevy 基本 ---
use bevy::{
    prelude::*
    , pbr::{
        CascadeShadowConfigBuilder
        , StandardMaterial
    }
    , render::{
        camera::{Camera, SubCameraView}
        , texture::ImagePlugin
        , primitives::Aabb
    }
    , window::PrimaryWindow
};

use crate::consts::app::*;
use crate::plugins::structs::components::PostProcessSettings;
use crate::plugins::post_process::PostProcessPlugin;

#[derive(Component)]
struct WindowCamera;

fn setup_window_camera(
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>
) {
    let mut display_full_size = UVec2::new(GAME_WIDTH as u32, GAME_HEIGHT as u32);
    let display_size = UVec2::new(GAME_WIDTH as u32, GAME_HEIGHT as u32);

    if let Some(window) = windows.iter_mut().next() {
        // ウィンドウを取得できた場合はディスプレイの解像度を最大画面サイズとする
        display_full_size = UVec2::new( window.resolution.physical_width() as u32, window.resolution.physical_height() as u32)
    }

    let bundle = (
        Camera3d::default()
        , Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y)
        , Projection::Perspective(PerspectiveProjection {
            fov: (45.0_f32).to_radians(),           // ４５° 視野角
            aspect_ratio: GAME_WIDTH / GAME_HEIGHT, // アスペクト比
            ..default()
        })
        , Camera {
                sub_camera_view: Some( SubCameraView {
                                           full_size: display_full_size
                                           , offset: Vec2::ZERO
                                           , size: display_size
                                       }
                                     )
                                     , order: 1
                                     , ..default()
               }
        , PostProcessSettings::default()
        , WindowCamera
    );

    commands.spawn(bundle);
}

//
// 物理レンダリングベースのライトを適用しないGLTFモデルの一覧リソース
//
#[derive(Default, Resource)]
struct UnlitGltfs(Vec<Entity>);

#[derive(Component)]
struct Cube;

//
// ゲームで使用するGLTFを読み込む
//
fn setup_load_gltf(
    mut commands: Commands
    , asset_server: Res<AssetServer>
    , mut unlit_gltfs: ResMut<UnlitGltfs>,
) {
    let bundle = (
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("glbs/cube_001.gltf")))
        , Transform::from_xyz(0.0, 0.0, 0.0)
        , GlobalTransform::default()
        , Cube
        , Aabb::default()
    );

    // unlit_gltfs.0.push(commands.spawn(bundle).id());
}

//
// シーンで使用するデフォルトのライトを設定する
//
fn setup_default_light(
    mut commands: Commands
) {
    // キーライト（全体を照らすメインとなる光源）
    let key_light = (
        Transform::from_rotation(
            Quat::from_euler( // クォータニオン オイラー回転 Z→Y→X順に回転
                EulerRot::YXZ
                , -45f32.to_radians() // ヨー　（Y軸回り）
                , -30f32.to_radians() // ピッチ（X軸回り）
                , 0.0                 // ロール（Z軸回り）
            )
        )
        , DirectionalLight {
            illuminance: 5000.0 // 光量
            , shadows_enabled: true
            , ..default()
        }
        , CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6, // 影を描画する距離
            ..default()
        }.build()
    );

    // リムライト（背面から照らす光）
    let rim_light = (
        Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            (180f32 - 45f32).to_radians() // キーの反対側寄り
            , 10f32.to_radians()          // ほぼ水平〜やや上
            , 0.0
        ))
        , DirectionalLight {
            illuminance: 1500.0
            , shadows_enabled: false
            , ..default()
        }
    );

    commands.spawn(key_light);
    commands.spawn(rim_light);
}

//
// UIの親（ルート）ノード
//
#[derive(Component)]
struct UiRoot;

//
// UIの親ノードをセットする
//
fn setup_ui_root(mut commands: Commands) {
    // UIの起点となる親ノードのバンドル
    let bundle = (
            Node {
                  width:  Val::Percent(100.0)
                , height: Val::Percent(100.0)
                , ..Default::default()
            }
            , BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0))
            , UiRoot
    );

    commands.spawn(bundle);
}

fn camera_rotation(
    input: Res<ButtonInput<MouseButton>>
    , mut transforms: ParamSet<(
        Query<&mut Transform, With<WindowCamera>>
        , Query<&mut Transform, With<Cube>>
    )>
) {
    if input.just_pressed(MouseButton::Left) {
        let angle = 0.1_f32;
        if let Ok(mut transform) = transforms.p0().single_mut() {
            println!("Camera: {:?}", transform.translation);
            transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, angle));
        }

        if let Ok(transform) = transforms.p1().single_mut() {
            println!("Model:  {:?}", transform.translation);
        }
    }
}

//
// モデルにライトの効果を適用しない
// ※ Bevy のデフォルトの三次元空間は物理ベースレンダリングなためライトは当たらないと描画されない
//    ただローポリゴンとは相性は良くないためライトを適用したくないモデルはこの処理を通す
//
fn materials_unlit(
    unlit_gltfs: Res<UnlitGltfs> // ライトを適用しないモデル（コンポーネント）のエンティティ一覧
    , added_materials: Query<(Entity, &MeshMaterial3d<StandardMaterial>), Added<MeshMaterial3d<StandardMaterial>>>
    , relation: Query<&ChildOf>
    , mut materials: ResMut<Assets<StandardMaterial>>
) {
    // １つも登録がなければ何もしない
    if unlit_gltfs.0.is_empty() {
        return;
    }

    // クエリから取得できたマテリアルがエンティティリソース一覧に登録されたものかどうかを判定する
    for (entity, added_material) in &added_materials {
        let mut current = Some(entity);
        let mut unlit_marked = false;
        while let Some(e) = current {
            if unlit_gltfs.0.contains(&e) {
                unlit_marked = true;
                break;
            }

            // 親もすべて辿り探す
            current = relation.get(e).ok().map(|child| child.parent());
         }

        if unlit_marked {
            let handle: &Handle<StandardMaterial> = &added_material.0;
            if let Some(mat) = materials.get_mut(handle) {
                mat.unlit = true;
                mat.emissive = Color::WHITE.into();
            }
        }
    }
}

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(CLEAR_WINDOW_COLOR))
       .insert_resource(AmbientLight { color: Color::WHITE, brightness: 1.0, affects_lightmapped_meshes: false }) // シーン全体に均等なライトを設定する
       .insert_resource(UnlitGltfs::default())
       .add_plugins((DefaultPlugins.set( WindowPlugin {
                                              primary_window: Some( Window {
                                                                        title: "Default".into(),
                                                                        name: Some("Default.app".into()),
                                                                        resolution: (GAME_WIDTH, GAME_HEIGHT).into(),
                                                                        position: WindowPosition::Centered(MonitorSelection::Primary),
                                                                        ..default()
                                                                    }
                                                                  )
                                                              , ..default()
                                          }
                                        )
                                        .set(ImagePlugin::default_nearest()) // 画像はすべて最近傍で補完する
                                        , PostProcessPlugin
                    ));

    app.add_systems(Startup, (
            setup_load_gltf
            , setup_default_light
            , setup_window_camera
            , setup_ui_root
        ))
       .add_systems(Update, (
            camera_rotation
            , materials_unlit
        )).run();
}
