use vizia::prelude::*;

use crate::ui::common::{hex_color, TRACKS};

pub struct TrackHeaders;

impl TrackHeaders {
    pub fn new(cx: &mut Context) -> Handle<'_, impl View> {
        VStack::new(cx, |cx| {
            // Ruler spacer to align with timeline ruler
            Element::new(cx).class("track-headers-ruler-spacer");

            // Track headers
            for track in TRACKS.iter() {
                let bg_color = hex_color(track.color, 255);
                let icon_bg = hex_color(track.color, 77); // ~30% alpha

                HStack::new(cx, move |cx| {
                    // Color bar
                    Element::new(cx)
                        .class("track-color-bar")
                        .background_color(bg_color);

                    // Icon
                    HStack::new(cx, |cx| {
                        Label::new(cx, track.icon);
                    }).class("track-icon")
                      .background_color(icon_bg);

                    // Name
                    Label::new(cx, track.name).class("track-name");

                    // M/S/R buttons
                    HStack::new(cx, |cx| {
                        Button::new(cx, |cx| Label::new(cx, "M"))
                            .class("btn-msr").class("btn-m")
                            .toggle_class("muted", track.muted);

                        Button::new(cx, |cx| Label::new(cx, "S"))
                            .class("btn-msr").class("btn-s")
                            .toggle_class("soloed", track.soloed);

                        Button::new(cx, |cx| Label::new(cx, "R"))
                            .class("btn-msr").class("btn-r")
                            .toggle_class("armed", track.armed);
                    }).class("track-buttons");
                }).class("track-header")
                  .toggle_class("selected", track.selected);
            }
        }).class("track-headers-panel")
    }
}
