use vizia::prelude::*;

use crate::ui::channel_strip::ChannelStrip;
use crate::ui::common::TRACKS;

pub struct MixerPanel;

impl MixerPanel {
    pub fn new(cx: &mut Context) -> Handle<'_, impl View> {
        VStack::new(cx, |cx| {
            // Mixer header
            HStack::new(cx, |cx| {
                Label::new(cx, "MIXER").class("mixer-header-label");
            }).class("mixer-header");

            // Channel strips
            HStack::new(cx, |cx| {
                for track in TRACKS.iter() {
                    ChannelStrip::new(
                        cx,
                        track.name,
                        track.color,
                        track.name == "Master",
                        track.selected,
                        track.muted,
                        track.soloed,
                        track.fader_pct,
                        track.level_pct,
                    );
                }
            }).class("mixer-channels");
        }).class("mixer-panel")
    }
}
