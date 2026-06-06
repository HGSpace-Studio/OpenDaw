use vizia::prelude::*;

use crate::ui::common::hex_color;

pub struct EditorPanel;

impl EditorPanel {
    pub fn new(cx: &mut Context) -> Handle<'_, impl View> {
        VStack::new(cx, |cx| {
            // Editor header with tabs
            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "Piano Roll");
                }).class("editor-tab").class("active");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Step Editor");
                }).class("editor-tab");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Score");
                }).class("editor-tab");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Audio");
                }).class("editor-tab");
            }).class("editor-header");

            // Editor content: Piano keys + Grid
            HStack::new(cx, |cx| {
                // Piano keys
                VStack::new(cx, |cx| {
                    let notes: &[(&str, &str, bool)] = &[
                        ("B4", "white", false),
                        ("A#4", "black", false),
                        ("A4", "white", false),
                        ("G#4", "black", false),
                        ("G4", "white", false),
                        ("F#4", "black", false),
                        ("F4", "white", false),
                        ("E4", "white", false),
                        ("D#4", "black", false),
                        ("D4", "white", false),
                        ("C#4", "black", false),
                        ("C4", "white", true),
                        ("B3", "white", false),
                        ("A#3", "black", false),
                        ("A3", "white", false),
                        ("G#3", "black", false),
                    ];

                    for &(note, key_type, is_c) in notes {
                        let is_white = key_type == "white";
                        let is_black = key_type == "black";
                        HStack::new(cx, move |cx| {
                            Label::new(cx, note).class("piano-key-label");
                        })
                        .class("piano-key")
                        .toggle_class("white", is_white)
                        .toggle_class("black", is_black)
                        .toggle_class("c-note", is_c);
                    }
                }).class("piano-keys");

                // Piano grid with MIDI notes
                VStack::new(cx, |cx| {
                    let row_types: &[&str] = &[
                        "white", "black", "white", "black", "white",
                        "black", "white", "white", "black", "white",
                        "black", "c-note", "white", "black", "white", "black",
                    ];

                    // MIDI note positions: (row_index, left_pct, width_pct, color)
                    let midi_notes: &[(usize, f32, f32, &str)] = &[
                        (0, 10.0, 15.0, "#bb6bd9"),
                        (2, 10.0, 15.0, "#bb6bd9"),
                        (4, 10.0, 8.0, "#bb6bd9"),
                        (4, 25.0, 8.0, "#bb6bd9"),
                        (6, 10.0, 15.0, "#bb6bd9"),
                        (7, 10.0, 15.0, "#bb6bd9"),
                        (9, 10.0, 8.0, "#bb6bd9"),
                        (9, 25.0, 8.0, "#bb6bd9"),
                        (11, 10.0, 30.0, "#6fcf97"),
                        (11, 50.0, 15.0, "#6fcf97"),
                        (12, 10.0, 15.0, "#bb6bd9"),
                        (14, 10.0, 30.0, "#6fcf97"),
                    ];

                    for (i, row_type) in row_types.iter().enumerate() {
                        let is_white = *row_type == "white";
                        let is_black = *row_type == "black";
                        let is_c = *row_type == "c-note";

                        // Collect notes for this row
                        let row_notes: Vec<(f32, f32, Color)> = midi_notes
                            .iter()
                            .filter(|(row_idx, _, _, _)| *row_idx == i)
                            .map(|&(_, left, width, color)| (left, width, hex_color(color, 204)))
                            .collect();

                        HStack::new(cx, move |cx| {
                            for (left_pct, width_pct, note_color) in row_notes {
                                Element::new(cx)
                                    .class("midi-note")
                                    .left(Percentage(left_pct))
                                    .width(Percentage(width_pct))
                                    .background_color(note_color);
                            }
                        })
                        .class("piano-row")
                        .toggle_class("white", is_white)
                        .toggle_class("black", is_black)
                        .toggle_class("c-note", is_c);
                    }
                }).class("piano-grid");
            }).class("editor-content");
        }).class("editor-panel")
    }
}
