use midi_file::midly::MidiMessage;
use piano_math::KeyboardRange;
use wgpu_glyph::GlyphBrush;
use wgpu_jumpstart::{TransformUniform, Uniform};

use crate::{config::Config, render::KeyboardRenderer, target::Target};

pub struct Keyboard {
    renderer: KeyboardRenderer,
}

fn get_layout(width: f32, height: f32) -> piano_math::KeyboardLayout {
    let range = piano_math::KeyboardRange::standard_88_keys();
    let white_count = range.white_count();
    let neutral_width = width / white_count as f32;
    let neutral_height = height * 0.2;

    piano_math::KeyboardLayout::from_range(neutral_width, neutral_height, range)
}

impl Keyboard {
    pub fn new(target: &Target) -> Self {
        let layout = get_layout(
            target.window_state.logical_size.width,
            target.window_state.logical_size.height,
        );

        let mut renderer = KeyboardRenderer::new(&target.gpu, &target.transform_uniform, layout);
        renderer.position_on_bottom_of_parent(target.window_state.logical_size.height);

        Self { renderer }
    }

    pub fn layout(&self) -> &piano_math::KeyboardLayout {
        self.renderer.layout()
    }

    fn set_layout(&mut self, layout: piano_math::KeyboardLayout) {
        self.renderer.set_layout(layout)
    }

    pub fn range(&self) -> &KeyboardRange {
        &self.layout().range
    }

    fn position_on_bottom_of_parent(&mut self, parent_height: f32) {
        self.renderer.position_on_bottom_of_parent(parent_height)
    }

    pub fn resize(&mut self, target: &Target) {
        let keyboard_layout = get_layout(
            target.window_state.logical_size.width,
            target.window_state.logical_size.height,
        );

        self.set_layout(keyboard_layout.clone());
        self.position_on_bottom_of_parent(target.window_state.logical_size.height);
    }

    pub fn update(&mut self, queue: &wgpu::Queue, brush: &mut GlyphBrush<()>) {
        self.renderer.update(queue, brush)
    }

    pub fn reset_notes(&mut self) {
        self.renderer.reset_notes()
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.renderer.render(transform_uniform, render_pass)
    }

    pub fn user_midi_event(&mut self, message: &MidiMessage) {
        let range_start = self.range().start() as usize;

        let (is_on, key) = match message {
            MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
            MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
            _ => return,
        };

        if self.range().contains(key) {
            let id = key as usize - range_start;
            let key = &mut self.renderer.key_states_mut()[id];

            key.set_pressed_by_user(is_on);
            self.renderer.queue_reupload();
        }
    }

    pub fn file_midi_events(&mut self, config: &Config, events: &[&midi_file::MidiEvent]) {
        let range_start = self.range().start() as usize;

        for e in events {
            let (is_on, key) = match e.message {
                MidiMessage::NoteOn { key, .. } => (true, key.as_int()),
                MidiMessage::NoteOff { key, .. } => (false, key.as_int()),
                _ => continue,
            };

            if self.range().contains(key) && e.channel != 9 {
                let id = key as usize - range_start;
                let key = &mut self.renderer.key_states_mut()[id];

                if is_on {
                    let color = &config.color_schema[e.track_color_id % config.color_schema.len()];
                    key.pressed_by_file_on(color);
                } else {
                    key.pressed_by_file_off();
                }

                self.renderer.queue_reupload();
            }
        }
    }
}