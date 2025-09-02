use std::borrow::Cow;
use bevy::{
    prelude::*
    , core_pipeline::{core_3d::graph::{Core3d, Node3d}}
    , render::{
        extract_component::{
            ExtractComponentPlugin
            , UniformComponentPlugin
        }
        , extract_resource::ExtractResourcePlugin
        , render_graph::{
            RenderGraphApp
            , ViewNodeRunner
        }
        , RenderApp
    }
};
use crate::consts::app::*;
use crate::plugins::structs::components::PostProcessSettings;
use crate::plugins::structs::post_processes::*;
use crate::plugins::functions::shader::rebuild_pipeline_when_shader_changes;

#[derive(Resource, Clone)]
pub struct PostProcessDefaults {
    pub shader_path: Cow<'static, str>
}
impl Default for PostProcessDefaults {
    fn default() -> Self {
        Self { shader_path: Cow::Borrowed(DEFAULT_SHADER_PATH) }
    }
}

//
// ポストプロセスを行うプラグイン
// 参考
// https://bevy.org/examples/shaders/custom-post-processing/
//
pub struct PostProcessPlugin;
impl Plugin for PostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PostProcessDefaults>();
        app.init_resource::<PostProcessShader>();
        app.add_plugins((
            ExtractComponentPlugin::<PostProcessSettings>::default()
            , UniformComponentPlugin::<PostProcessSettings>::default()
            , ExtractResourcePlugin::<PostProcessShader>::default()
        ));

        let shader = app.world().resource::<PostProcessShader>().clone();
        // We need to get the render app from the main app
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.insert_resource(shader);

            render_app
                .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(
                    Core3d
                    , PostProcessLabel
                )
                .add_render_graph_edges(
                    Core3d,
                    (
                        Node3d::Tonemapping
                        , PostProcessLabel
                        , Node3d::EndMainPassPostProcessing
                    )
                    ,
                )
                .add_systems(
                        bevy::render::Render
                        , rebuild_pipeline_when_shader_changes
                );
        } else {
            return;
        }
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<PostProcessPipeline>();
    }
}
