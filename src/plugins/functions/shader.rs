use bevy::{
    prelude::*
    , core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state
    , render::render_resource::*
};
use crate::plugins::structs::post_processes::*;

//
// PostProcessShader の値を変更された際に変更されたシェーダーに切り替える
//
pub fn rebuild_pipeline_when_shader_changes(
    shader_resource: Res<PostProcessShader>
    , mut pipeline: ResMut<PostProcessPipeline>
    , cache: ResMut<PipelineCache>,
) {
    // 変更がない場合は何もしない
    if !shader_resource.is_changed() { return; }

    // シェーダーが同じなら何もしない
    if pipeline.shader_handle == shader_resource.0 { return; }

    // 新しいシェーダーでパイプラインを再キュー
    let new_id = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("post_process_pipeline".into())
        , layout: vec![pipeline.layout.clone()]
        , vertex: fullscreen_shader_vertex_state()
        , fragment: Some(FragmentState {
            shader: shader_resource.0.clone() // 新ハンドル
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
    });

    pipeline.pipeline_id = new_id;
    pipeline.shader_handle = shader_resource.0.clone();
}
