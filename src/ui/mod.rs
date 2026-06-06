use vizia::prelude::*;

mod common;
mod transport;
mod inspector;
mod track_headers;
mod timeline;
mod channel_strip;
mod mixer_panel;
mod editor_panel;

use transport::TransportBar;
use inspector::InspectorPanel;
use track_headers::TrackHeaders;
use timeline::TimelinePanel;
use mixer_panel::MixerPanel;
use editor_panel::EditorPanel;

pub fn run() -> Result<(), String> {
    Application::new(|cx| {
        // Load Logic Pro style theme
        cx.add_stylesheet(include_style!("src/ui/theme.css"))
            .expect("Failed to load theme");

        // Root container
        VStack::new(cx, |cx| {
            // Top: Transport bar
            TransportBar::new(cx);

            // Middle: Inspector + Track area
            HStack::new(cx, |cx| {
                // Left: Inspector panel
                InspectorPanel::new(cx);

                // Center: Track headers + Timeline
                HStack::new(cx, |cx| {
                    // Track headers
                    TrackHeaders::new(cx);

                    // Timeline / arrangement area
                    TimelinePanel::new(cx);
                }).height(Stretch(1.0));
            }).height(Stretch(1.0));

            // Bottom: Editor panel
            EditorPanel::new(cx);

            // Bottom: Mixer panel
            MixerPanel::new(cx);
        }).class("app-root");
    })
    .title("OpenDAW")
    .inner_size((1440, 900))
    .run()
    .map_err(|err| err.to_string())
}
