use vizia::prelude::*;

use crate::ui::common::hex_color;

pub struct ChannelStrip;

impl ChannelStrip {
    pub fn new<'a>(
        cx: &'a mut Context,
        name: &'static str,
        color: &'static str,
        is_master: bool,
        is_selected: bool,
        muted: bool,
        soloed: bool,
        fader_pct: f32,
        level_pct: f32,
    ) -> Handle<'a, impl View> {
        let color_bar = hex_color(color, 255);

        VStack::new(cx, move |cx| {
            // Top color indicator bar
            Element::new(cx)
                .width(Stretch(1.0))
                .height(Pixels(3.0))
                .background_color(color_bar);

            // Effect slots
            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "EQ");
                }).class("channel-effect-slot");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Comp");
                }).class("channel-effect-slot");
            }).horizontal_gap(Pixels(2.0));

            // Pan knob area
            HStack::new(cx, |cx| {
                Label::new(cx, "C").class("channel-pan-label");
                // Pan knob placeholder
                Element::new(cx)
                    .width(Pixels(20.0))
                    .height(Pixels(20.0))
                    .corner_radius(Pixels(10.0))
                    .background_color(Color::rgb(58, 58, 60))
                    .border_color(Color::rgb(78, 78, 80))
                    .border_width(Pixels(1.0));
                Label::new(cx, "").class("channel-pan-label");
            }).class("channel-pan-area");

            // Fader + Level meter area
            HStack::new(cx, |cx| {
                // Level meter
                VStack::new(cx, |cx| {
                    Element::new(cx)
                        .class("level-meter-green")
                        .height(Percentage(level_pct * 70.0));
                }).class("level-meter");

                // Fader
                VStack::new(cx, |cx| {
                    Element::new(cx)
                        .class("fader-fill")
                        .height(Percentage(fader_pct * 100.0));
                    Element::new(cx)
                        .class("fader-thumb")
                        .bottom(Percentage(fader_pct * 100.0));
                }).class("fader-track");
            }).class("channel-fader-area");

            // M/S buttons
            HStack::new(cx, |cx| {
                if muted {
                    Button::new(cx, |cx| Label::new(cx, "M"))
                        .class("channel-btn-ms").class("muted");
                } else {
                    Button::new(cx, |cx| Label::new(cx, "M"))
                        .class("channel-btn-ms");
                }

                if soloed {
                    Button::new(cx, |cx| Label::new(cx, "S"))
                        .class("channel-btn-ms").class("soloed");
                } else {
                    Button::new(cx, |cx| Label::new(cx, "S"))
                        .class("channel-btn-ms");
                }
            }).class("channel-ms-buttons");

            // Track name
            Label::new(cx, name).class("channel-name");

            // dB display
            let db = if fader_pct >= 1.0 {
                "0.0".to_string()
            } else {
                let db_val = 20.0 * (fader_pct * 0.8 + 0.2).log10();
                format!("{:.1}", db_val)
            };
            Label::new(cx, db).class("channel-db");
        })
        .class("channel-strip")
        .toggle_class("master", is_master)
        .toggle_class("selected", is_selected)
    }
}
