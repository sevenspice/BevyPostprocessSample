use bevy::{
    prelude::*
    , asset::Handle
    , core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state
    , ecs::query::QueryItem
    , render::{
        extract_component::{
            ComponentUniforms
            , DynamicUniformIndex
        }
        , extract_resource::ExtractResource
        , render_graph::{
            NodeRunError
            , RenderGraphContext
            , RenderLabel
            , ViewNode
        }
        , render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer}
            , *
        }
        , renderer::{RenderContext, RenderDevice}
        , view::{ViewTarget}
    }
};

use crate::plugins::structs::components::PostProcessSettings;
use crate::plugins::post_process::PostProcessDefaults;

// ポストプロセスのどのシェーダーを使うかを持つリソース
#[derive(Resource, Clone, ExtractResource)]
pub struct PostProcessShader(pub Handle<Shader>);

// Bevy でのデフォルト初期化トレイト
// リソースから PostProcessDefaults を取り出して保持しているシェーダーのパスで初期化する
impl FromWorld for PostProcessShader {
    fn from_world(world: &mut World) -> Self {
        let defaults = world.get_resource::<PostProcessDefaults>()
            .cloned()
            .unwrap_or_default();

        let asset_server = world.resource::<AssetServer>().clone();
        let handle = asset_server.load(defaults.shader_path.as_ref());
        PostProcessShader(handle)
    }
}

//
// レンダリングパイプラインを保持するリソース
//
#[derive(Resource)]
pub struct PostProcessPipeline {
    pub layout: BindGroupLayout
    , pub sampler: Sampler
    , pub pipeline_id: CachedRenderPipelineId
    , pub shader_handle: Handle<Shader>
}
impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let (layout, sampler, shader_handle) = {
            let render_device   = world.resource::<RenderDevice>();
            let shader_resource = world.resource::<PostProcessShader>();
            let layout = render_device.create_bind_group_layout(
                "post_process_bind_group_layout",
                &BindGroupLayoutEntries::sequential(
                    ShaderStages::FRAGMENT,
                    (

                        texture_2d(TextureSampleType::Float { filterable: true })
                        , sampler(SamplerBindingType::Filtering)
                        , uniform_buffer::<PostProcessSettings>(true)
                    ),
                )
            );

            let sampler = render_device.create_sampler(&SamplerDescriptor::default());
            // let shader = world.load_asset("");
            (layout, sampler, shader_resource.0.clone())
        };

        let pipeline_id = {
            let cache = world.resource_mut::<PipelineCache>();
            cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("post_process_pipeline".into())
                , layout: vec![layout.clone()]
                , vertex: fullscreen_shader_vertex_state()
                , fragment: Some(FragmentState {
                    shader: shader_handle.clone()
                    , shader_defs: vec![]
                    , entry_point: "fragment".into()
                    , targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default()
                        , blend: None
                        , write_mask: ColorWrites::ALL
                    })]
                })
                , primitive: PrimitiveState::default()
                , depth_stencil: None
                , multisample: MultisampleState::default()
                , push_constant_ranges: vec![]
                , zero_initialize_workgroup_memory: false
            })
        };

        Self {
            layout
            , sampler
            , pipeline_id
            , shader_handle
        }
    }
}

//
// ポストプロセスを識別するためのラベル
//
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct PostProcessLabel;

//
// ポストプロセスのレンダーパイプラインノードの定義
//
#[derive(Default)]
pub struct PostProcessNode;
impl ViewNode for PostProcessNode {
    type ViewQuery = (
        &'static ViewTarget
        , &'static PostProcessSettings
        , &'static DynamicUniformIndex<PostProcessSettings>
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _post_process_settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<PostProcessPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };
        let settings_uniforms = world.resource::<ComponentUniforms<PostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };
        let post_process = view_target.post_process_write();
        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group"
            , &post_process_pipeline.layout
            , &BindGroupEntries::sequential((
                post_process.source
                , &post_process_pipeline.sampler
                , settings_binding.clone()
            ))
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination
                , resolve_target: None
                , ops: Operations::default()
            })]
            , depth_stencil_attachment: None
            , timestamp_writes: None
            , occlusion_query_set: None
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
