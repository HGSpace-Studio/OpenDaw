use vizia::prelude::*;

pub struct InspectorPanel;

impl InspectorPanel {
    pub fn new(cx: &mut Context) -> Handle<'_, impl View> {
        VStack::new(cx, |cx| {
            // Inspector header
            HStack::new(cx, |cx| {
                Label::new(cx, "INSPECTOR").class("inspector-header-label");
            }).class("inspector-header");

            // Track properties section
            VStack::new(cx, |cx| {
                // Track name
                HStack::new(cx, |cx| {
                    Label::new(cx, "Kick Drum").class("inspector-track-name-text");
                }).class("inspector-track-name");

                // Track type
                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{1F3B5}").class("inspector-track-type-icon");
                    Label::new(cx, "Audio").class("inspector-track-type-label");
                }).class("inspector-track-type");

                // Volume & Pan
                HStack::new(cx, |cx| {
                    Label::new(cx, "Volume").class("inspector-label");
                    Label::new(cx, "-2.4 dB").class("inspector-value");
                }).class("inspector-row");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Pan").class("inspector-label");
                    Label::new(cx, "C").class("inspector-value");
                }).class("inspector-row");
            }).class("inspector-section");

            // Insert Effects section
            VStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{25BC}").class("inspector-section-arrow");
                    Label::new(cx, "INSERTS").class("inspector-section-title");
                }).class("inspector-section-header");

                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{2699}").class("inspector-slot-icon");
                    Label::new(cx, "Channel EQ").class("inspector-slot-name");
                    Label::new(cx, "BYP").class("inspector-slot-bypass");
                }).class("inspector-slot");

                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{2699}").class("inspector-slot-icon");
                    Label::new(cx, "Compressor").class("inspector-slot-name");
                    Label::new(cx, "").class("inspector-slot-bypass");
                }).class("inspector-slot");

                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{2699}").class("inspector-slot-icon");
                    Label::new(cx, "No Plug-in").class("inspector-slot-name");
                    Label::new(cx, "").class("inspector-slot-bypass");
                }).class("inspector-slot");

                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{2699}").class("inspector-slot-icon");
                    Label::new(cx, "No Plug-in").class("inspector-slot-name");
                    Label::new(cx, "").class("inspector-slot-bypass");
                }).class("inspector-slot");
            }).class("inspector-section");

            // Send Effects section
            VStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{25BC}").class("inspector-section-arrow");
                    Label::new(cx, "SENDS").class("inspector-section-title");
                }).class("inspector-section-header");

                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{2192}").class("inspector-slot-icon");
                    Label::new(cx, "Reverb").class("inspector-slot-name");
                    Label::new(cx, "-4.2").class("inspector-slot-bypass");
                }).class("inspector-slot");

                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{2192}").class("inspector-slot-icon");
                    Label::new(cx, "Delay").class("inspector-slot-name");
                    Label::new(cx, "-inf").class("inspector-slot-bypass");
                }).class("inspector-slot");
            }).class("inspector-section");

            // Region Properties section
            VStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "\u{25BC}").class("inspector-section-arrow");
                    Label::new(cx, "REGION").class("inspector-section-title");
                }).class("inspector-section-header");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Name").class("inspector-label");
                    Label::new(cx, "Kick Main").class("inspector-value");
                }).class("inspector-row");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Quantize").class("inspector-label");
                    Label::new(cx, "1/16").class("inspector-value");
                }).class("inspector-row");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Transpose").class("inspector-label");
                    Label::new(cx, "0").class("inspector-value");
                }).class("inspector-row");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Speed").class("inspector-label");
                    Label::new(cx, "1.0x").class("inspector-value");
                }).class("inspector-row");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Fade In").class("inspector-label");
                    Label::new(cx, "0.0 ms").class("inspector-value");
                }).class("inspector-row");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Fade Out").class("inspector-label");
                    Label::new(cx, "0.0 ms").class("inspector-value");
                }).class("inspector-row");
            }).class("inspector-section");

            // Spacer
            Element::new(cx).class("flex-spacer");
        }).class("inspector")
    }
}
