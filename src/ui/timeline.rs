use vizia::prelude::*;

use crate::ui::common::{hex_color, TRACKS};

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn new(cx: &mut Context) -> Handle<'_, impl View> {
        VStack::new(cx, |cx| {
            // Ruler
            HStack::new(cx, |cx| {
                for bar in 1..=16 {
                    Label::new(cx, format!("{:03}", bar)).class("ruler-mark");
                }
            }).class("ruler");

            // Track lanes container with playhead
            VStack::new(cx, |cx| {
                for (i, track) in TRACKS.iter().enumerate() {
                    let is_even = i % 2 == 0;
                    let region_color = hex_color(track.color, 128); // ~50% alpha
                    let border_color = hex_color(track.color, 255);

                    HStack::new(cx, move |cx| {
                        for &(left_pct, width_pct) in track.regions {
                            Element::new(cx)
                                .class("region")
                                .background_color(region_color)
                                .border_color(border_color)
                                .left(Percentage(left_pct))
                                .width(Percentage(width_pct));
                        }
                    }).class("track-lane")
                      .toggle_class("even", is_even)
                      .toggle_class("odd", !is_even);
                }

                // Playhead - white vertical line at bar 2 position
                Element::new(cx)
                    .class("playhead")
                    .left(Percentage(12.5));
            }).class("track-lanes");
        }).class("timeline-panel")
    }
}
