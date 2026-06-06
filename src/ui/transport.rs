use vizia::prelude::*;

pub struct TransportBar;

impl TransportBar {
    pub fn new(cx: &mut Context) -> Handle<'_, impl View> {
        HStack::new(cx, |cx| {
            // Mode buttons group
            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "CYCLE"))
                    .class("btn-mode");
                Button::new(cx, |cx| Label::new(cx, "DROP"))
                    .class("btn-mode");
            }).class("transport-group");

            // Separator
            Element::new(cx).class("transport-separator");

            // Transport controls
            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "\u{23EE}"))
                    .class("btn-transport");
                Button::new(cx, |cx| Label::new(cx, "\u{23ED}"))
                    .class("btn-transport");
                Button::new(cx, |cx| Label::new(cx, "\u{25B6}"))
                    .class("btn-play");
                Button::new(cx, |cx| Label::new(cx, "\u{23F8}"))
                    .class("btn-transport");
                Button::new(cx, |cx| Label::new(cx, "\u{23FA}"))
                    .class("btn-rec");
            }).class("transport-group");

            // Separator
            Element::new(cx).class("transport-separator");

            // Position display
            HStack::new(cx, |cx| {
                Label::new(cx, "001").class("position-segment");
                Label::new(cx, "|").class("position-sep");
                Label::new(cx, "02").class("position-segment");
                Label::new(cx, "|").class("position-sep");
                Label::new(cx, "000").class("position-segment");
            }).class("position-display");

            // Separator
            Element::new(cx).class("transport-separator");

            // BPM
            Label::new(cx, "120.000").class("bpm-display");

            // Time signature
            Label::new(cx, "4/4").class("time-sig-display");

            // Spacer
            Element::new(cx).class("flex-spacer");

            // CPU / Disk display
            HStack::new(cx, |cx| {
                Label::new(cx, "CPU").class("cpu-label");
                VStack::new(cx, |cx| {
                    Element::new(cx).class("cpu-bar-fill");
                }).class("cpu-bar-bg");
                Label::new(cx, "HD").class("cpu-label");
                VStack::new(cx, |cx| {
                    Element::new(cx).class("cpu-bar-fill").class("hd");
                }).class("cpu-bar-bg");
            }).class("cpu-display");
        }).class("transport-bar")
    }
}
