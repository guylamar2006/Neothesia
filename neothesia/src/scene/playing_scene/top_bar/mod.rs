use std::time::{Duration, Instant};

use neothesia_core::render::QuadInstance;
use ui::{LooperMsg, ProgressBarMsg};
use wgpu_jumpstart::Color;

use crate::{context::Context, NeothesiaEvent};

use super::{
    animation::{Animated, Easing},
    rewind_controller::RewindController,
    PlayingScene, LAYER_FG,
};

mod renderer;
pub mod ui;
mod widget;

use renderer::NuonRenderer;

pub struct TopBar {
    animation: Animated<bool, Instant>,
    is_expanded: bool,

    settings_animation: Animated<bool, Instant>,

    settings_active: bool,

    looper_active: bool,
    loop_start: Duration,
    loop_end: Duration,

    ui: ui::Ui,
}

impl TopBar {
    pub fn new() -> Self {
        Self {
            animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),
            settings_animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),

            is_expanded: false,
            settings_active: false,

            looper_active: false,
            loop_start: Duration::ZERO,
            loop_end: Duration::ZERO,

            ui: ui::Ui::new(),
        }
    }

    pub fn is_looper_active(&self) -> bool {
        self.looper_active
    }

    pub fn loop_start_timestamp(&self) -> Duration {
        self.loop_start
    }

    pub fn loop_end_timestamp(&self) -> Duration {
        self.loop_end
    }

    fn on_msg(scene: &mut PlayingScene, ctx: &mut Context, msg: &ui::Msg) {
        use ui::Msg;
        match msg {
            Msg::PauseResume => {
                scene.player.pause_resume();
            }
            Msg::SpeedUp => {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() + 0.1);
            }
            Msg::SpeedDown => {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() - 0.1);
            }
            Msg::SettingsToggle => {
                scene.top_bar.settings_active = !scene.top_bar.settings_active;
            }
            Msg::GoBack => {
                ctx.proxy
                    .send_event(NeothesiaEvent::MainMenu(Some(scene.player.song().clone())))
                    .ok();
            }
            Msg::Looper(msg) => match msg {
                LooperMsg::Toggle => {
                    scene.top_bar.looper_active = !scene.top_bar.looper_active;

                    // Looper enabled for the first time
                    if scene.top_bar.looper_active
                        && scene.top_bar.loop_start.is_zero()
                        && scene.top_bar.loop_end.is_zero()
                    {
                        scene.top_bar.loop_start = scene.player.time();
                        scene.top_bar.loop_end = scene.player.time() + Duration::from_secs(5);
                    }
                }
                LooperMsg::MoveStart(t) => {
                    scene.top_bar.loop_start = *t;
                }
                LooperMsg::MoveEnd(t) => {
                    scene.top_bar.loop_end = *t;
                }
            },
            Msg::ProggresBar(msg) => {
                let PlayingScene {
                    player,
                    keyboard,
                    rewind_controller,
                    ..
                } = scene;

                match msg {
                    ProgressBarMsg::Pressed => {
                        if !rewind_controller.is_rewinding() {
                            rewind_controller.start_mouse_rewind(player);

                            let x = ctx.window_state.cursor_logical_position.x;
                            let w = ctx.window_state.logical_size.width;

                            let p = x / w;
                            player.set_percentage_time(p);
                            keyboard.reset_notes();
                        }
                    }
                    ProgressBarMsg::Released => {
                        if let RewindController::Mouse { .. } = rewind_controller {
                            rewind_controller.stop_rewind(player);
                        }
                    }
                }
            }
        }
    }

    #[profiling::function]
    fn update_nuon(scene: &mut PlayingScene, ctx: &mut Context, _delta: Duration, y: f32) {
        let mut root = scene
            .top_bar
            .ui
            .view(ui::UiData {
                y,
                is_settings_open: scene.top_bar.settings_active,
                is_looper_on: scene.top_bar.is_looper_active(),
                speed: ctx.config.speed_multiplier(),
                player: &scene.player,
                loop_start: scene.top_bar.loop_start_timestamp(),
                loop_end: scene.top_bar.loop_end_timestamp(),
            })
            .into();

        let layout = {
            profiling::scope!("nuon_layout");
            root.as_widget_mut().layout(&nuon::LayoutCtx {
                x: 0.0,
                y: 0.0,
                w: ctx.window_state.logical_size.width,
                h: ctx.window_state.logical_size.height,
            })
        };

        let mut messages = vec![];

        scene
            .nuon_event_queue
            .dispatch_events(&mut messages, root.as_widget_mut(), &layout);

        {
            profiling::scope!("nuon_render");
            root.as_widget().render(
                &mut NuonRenderer {
                    quads: &mut scene.quad_pipeline,
                    text: &mut ctx.text_renderer,
                },
                &layout,
                &nuon::RenderCtx {},
            );
        }

        drop(root);

        for msg in messages.iter() {
            Self::on_msg(scene, ctx, msg);
        }
    }

    #[profiling::function]
    pub fn update(scene: &mut PlayingScene, ctx: &mut Context, delta: Duration) {
        let PlayingScene { top_bar, .. } = scene;

        let window_state = &ctx.window_state;

        let h = 75.0;
        let is_hovered = window_state.cursor_logical_position.y < h * 1.7;

        top_bar.is_expanded = is_hovered;
        top_bar.is_expanded |= top_bar.settings_active;
        top_bar.is_expanded |= top_bar.ui.looper.is_grabbed();
        top_bar.is_expanded |= top_bar.ui.proggress_bar.is_grabbed();

        let now = ctx.frame_timestamp;

        top_bar.animation.transition(top_bar.is_expanded, now);
        top_bar
            .settings_animation
            .transition(top_bar.settings_active, now);

        let y = top_bar.animation.animate_bool(-h + 5.0, 0.0, now);

        update_settings_card(scene, ctx, y);
        Self::update_nuon(scene, ctx, delta, y);
    }
}

fn update_settings_card(scene: &mut PlayingScene, ctx: &mut Context, y: f32) {
    let PlayingScene {
        top_bar,
        quad_pipeline,
        ..
    } = scene;

    let h = 75.0;
    let w = ctx.window_state.logical_size.width;
    let now = ctx.frame_timestamp;

    if top_bar.settings_animation.in_progress(now) || top_bar.settings_animation.value {
        let card_w = 300.0;
        let card_x = top_bar.settings_animation.animate_bool(card_w, 0.0, now);

        let bar_bg: Color = Color::from_rgba8(37, 35, 42, 1.0);
        quad_pipeline.push(
            LAYER_FG,
            QuadInstance {
                position: [card_x + w - card_w, y + h + 1.0],
                size: [card_w, 100.0],
                color: bar_bg.into_linear_rgba(),
                border_radius: [10.0, 0.0, 10.0, 0.0],
            },
        );
    }
}
