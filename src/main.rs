use eframe::egui;
use serde_json::{self, Value};

use egui::TextStyle;
use egui::text::{LayoutJob, TextFormat};
use egui::{CollapsingHeader, Color32,Rounding,Ui,TextEdit};
// use std::process::{Command, Stdio}; // For process command

fn create_highlighted_layout_sections(
    ui: &egui::Ui,
    full_text_content: &str, // Renamed to clearly indicate it's the full content
    search_query: &str,
    default_text_color: Color32,
    is_strong: bool,
) -> (String, Vec<egui::text::LayoutSection>) {
    // Returns a tuple: (full_text, sections)
    let mut sections = Vec::new();
    // let current_text_byte_offset = 0; // Tracks byte offset for Ranges

    let base_font_id = TextStyle::Body.resolve(ui.style());

    if is_strong {
        //     base_font_id.weight = FontWeight::Bold;
    }

    let default_format = TextFormat {
        font_id: base_font_id.clone(),
        extra_letter_spacing: 0.0, // Add this line
        line_height: None,         // Add this line
        color: default_text_color,
        italics: false,
        strikethrough: Default::default(), // Use Default for Stroke
        underline: Default::default(),     // Use Default for Stroke
        background: Color32::TRANSPARENT,
        valign: egui::Align::Center,
    };

    let highlighted_format = TextFormat {
        font_id: base_font_id.clone(),
        extra_letter_spacing: 0.0, // Add this line
        line_height: None,         // Add this line
        color: Color32::BLACK,
        background: Color32::from_rgb(255, 255, 0),
        italics: false,
        strikethrough: Default::default(),
        underline: Default::default(),
        valign: egui::Align::Center,
    };

    if search_query.is_empty() {
        // If no search query, the entire text is one section with default format
        sections.push(egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..full_text_content.len(),
            format: default_format,
        });
        return (full_text_content.to_string(), sections);
    }

    let lower_full_text = full_text_content.to_lowercase();
    let lower_search_query = search_query.to_lowercase();
    let mut last_end_byte = 0;

    for (start_byte, matched_str) in lower_full_text.match_indices(&lower_search_query) {
        let matched_len_bytes = matched_str.len(); // Get the byte length of the matched string slice

        // Add the part before the match (if any)
        if start_byte > last_end_byte {
            sections.push(egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: last_end_byte..start_byte,
                format: default_format.clone(),
            });
        }

        // Add the matched part with highlight
        sections.push(egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: start_byte..(start_byte + matched_len_bytes), // Use the byte length here
            format: highlighted_format.clone(),
        });

        last_end_byte = start_byte + matched_len_bytes;
    }

    // Add the remaining part after the last match (if any)
    if last_end_byte < full_text_content.len() {
        sections.push(egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: last_end_byte..full_text_content.len(),
            format: default_format,
        });
    }

    (full_text_content.to_string(), sections)
}

fn parse_json_to_value(json_string: &str) -> Result<Value, String> {
    // Attempt to parse the input string into a serde_json::Value.
    // This will validate the JSON.
    let parsed_json: Result<Value, serde_json::Error> = serde_json::from_str(json_string);

    match parsed_json {
        Ok(value) => Ok(value),
        Err(e) => {
            // If parsing fails, it means the input is not valid JSON.
            // Return an error message indicating the problem.
            Err(format!("Invalid JSON input: {}", e))
        }
    }
}

fn render_json_value(
    ui: &mut egui::Ui,
    key_name: Option<&str>,
    value: &Value,
    path_segment: &str,
    search_query: &str,
) {
    ui.horizontal(|ui| {
        // let mut key_text = String::new();
        // Display the key name if provided (for object fields)
        if let Some(key) = key_name {
            // key_text = format!("\"{}\":", key);
            // let mut rich_key_text = RichText::new(key_text.clone())
            //     .strong()
            //     .color(Color32::LIGHT_BLUE);
            // if !search_query.is_empty() && key.to_lowercase().contains(&search_query.to_lowercase())
            // {
            //     rich_key_text = rich_key_text.color(Color32::BLACK); // Yellow highlight
            //     rich_key_text = rich_key_text.background_color(Color32::YELLOW); // Yellow highlight
            // }
            // ui.add(egui::Label::new(rich_key_text));
            // ui.add_space(5.0);
            let key_display = format!("\"{}\":", key);
            // Get the full text and its sections
            let (full_text, layout_sections) = create_highlighted_layout_sections(
                ui,
                &key_display,
                search_query,
                Color32::LIGHT_BLUE,
                true,
            );
            // Create LayoutJob from the full text and the sections
            let layout_job = LayoutJob {
                text: full_text,           // The entire string for the job
                sections: layout_sections, // The formatting sections
                // wrap: true, // Typically want wrapping for labels
                ..Default::default()
            };
            ui.add(egui::Label::new(egui::WidgetText::LayoutJob(layout_job)));
            ui.add_space(5.0);
        }

        match value {
            Value::Object(map) => {
                // Header for objects
                let header_text = format!("{{ ... }} ({} items)", map.len());
                // Create a unique ID for this collapsing header
                let id = ui.make_persistent_id(format!("{}_object", path_segment));

                CollapsingHeader::new(header_text)
                    .id_salt(id)
                    .default_open(true) // Starts open by default
                    .show(ui, |ui| {
                        // Indent the content within the collapsing header
                        ui.indent("object_indent", |ui| {
                            for (key, val) in map {
                                // Recursively render each key-value pair in the object
                                render_json_value(
                                    ui,
                                    Some(key),
                                    val,
                                    &format!("{}.{}", path_segment, key),
                                    search_query,
                                );
                            }
                        });
                    });
            }
            Value::Array(arr) => {
                // Header for arrays
                let header_text = format!("[ ... ] ({} items)", arr.len());
                // Create a unique ID for this collapsing header
                let id = ui.make_persistent_id(format!("{}_array", path_segment));

                CollapsingHeader::new(header_text)
                    .id_salt(id)
                    .default_open(true) // Starts open by default
                    .show(ui, |ui| {
                        // Indent the content within the collapsing header
                        ui.indent("array_indent", |ui| {
                            for (index, val) in arr.iter().enumerate() {
                                // Recursively render each element in the array.
                                // We pass `None` for key_name as array elements don't have keys,
                                // but the index is part of the unique path.
                                render_json_value(
                                    ui,
                                    None,
                                    val,
                                    &format!("{}[{}]", path_segment, index),
                                    search_query,
                                );
                            }
                        });
                    });
            }
            // Handle primitive JSON types
            Value::String(s) => {
                // key_text = format!("\"{}\"", s);
                // let mut rich_key_text = RichText::new(key_text.clone())
                //     .strong()
                //     .color(Color32::GREEN);
                // if !search_query.is_empty() && s.to_lowercase().contains(&search_query.to_lowercase())
                // {
                //     rich_key_text = rich_key_text.color(Color32::BLACK); // Yellow highlight
                //     rich_key_text = rich_key_text.background_color(Color32::YELLOW); // Yellow highlight
                // }
                // ui.add(egui::Label::new(rich_key_text));
                // ui.add_space(5.0);
                let (full_text, layout_sections) = create_highlighted_layout_sections(
                    ui,
                    &format!("\"{}\"", s),
                    search_query,
                    Color32::GREEN,
                    false,
                );
                let layout_job = LayoutJob {
                    text: full_text,
                    sections: layout_sections,
                    // wrap: true,
                    ..Default::default()
                };
                ui.label(egui::WidgetText::LayoutJob(layout_job));
            }
            Value::Number(n) => {
                // let num_str = n.to_string(); // Immutable copy for search check
                // let mut rich_key_text = RichText::new(num_str.clone())
                //     .strong()
                //     .color(Color32::YELLOW);
                // if !search_query.is_empty() && num_str.to_lowercase().contains(&search_query.to_lowercase())
                // {
                //     rich_key_text = rich_key_text.color(Color32::BLACK); // Yellow highlight
                //     rich_key_text = rich_key_text.background_color(Color32::YELLOW); // Yellow highlight
                // }
                // ui.add(egui::Label::new(rich_key_text));
                // ui.add_space(5.0);
                let (full_text, layout_sections) = create_highlighted_layout_sections(
                    ui,
                    &n.to_string(),
                    search_query,
                    Color32::YELLOW,
                    false,
                );
                let layout_job = LayoutJob {
                    text: full_text,
                    sections: layout_sections,
                    // wrap: true,
                    ..Default::default()
                };
                ui.label(egui::WidgetText::LayoutJob(layout_job));
            }
            Value::Bool(b) => {
                // let bool_str = b.to_string(); // Immutable copy for search check
                // let mut rich_key_text = RichText::new(bool_str.clone())
                //     .strong()
                //     .color(Color32::KHAKI);
                // if !search_query.is_empty() && bool_str.to_lowercase().contains(&search_query.to_lowercase())
                // {
                //     rich_key_text = rich_key_text.color(Color32::BLACK); // Yellow highlight
                //     rich_key_text = rich_key_text.background_color(Color32::YELLOW); // Yellow highlight
                // }
                // ui.add(egui::Label::new(rich_key_text));
                // ui.add_space(5.0);
                let (full_text, layout_sections) = create_highlighted_layout_sections(
                    ui,
                    &b.to_string(),
                    search_query,
                    Color32::KHAKI,
                    false,
                );
                let layout_job = LayoutJob {
                    text: full_text,
                    sections: layout_sections,
                    // wrap: true,
                    ..Default::default()
                };
                ui.label(egui::WidgetText::LayoutJob(layout_job));
            }
            Value::Null => {
                // let null_str = "null".to_string(); // Immutable copy for search check
                // let mut rich_key_text = RichText::new(null_str.clone())
                //     .strong()
                //     .color(Color32::DARK_GRAY);
                // if !search_query.is_empty() && null_str.to_lowercase().contains(&search_query.to_lowercase())
                // {
                //     rich_key_text = rich_key_text.color(Color32::BLACK); // Yellow highlight
                //     rich_key_text = rich_key_text.background_color(Color32::YELLOW); // Yellow highlight
                // }
                // ui.add(egui::Label::new(rich_key_text));
                // ui.add_space(5.0);
                let (full_text, layout_sections) = create_highlighted_layout_sections(
                    ui,
                    "null",
                    search_query,
                    Color32::DARK_GRAY,
                    false,
                );
                let layout_job = LayoutJob {
                    text: full_text,
                    sections: layout_sections,
                    // wrap: true,
                    ..Default::default()
                };
                ui.label(egui::WidgetText::LayoutJob(layout_job));
            }
        }
    });
}

// fn main() -> Result<(), eframe::Error> {
//     // Configure the options for our eframe application window.
//     // We set the initial window size to be 320x240 logical pixels.
//     let options = eframe::NativeOptions {
//         viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(320.0, 240.0)),
//         ..Default::default() // Use default values for other options.
//     };

//     // Run the egui application.
//     // `eframe::run_native` takes the application name, NativeOptions,
//     // and a closure that initializes our custom egui application struct.
//     // The closure takes a `CreationContext` which provides access to egui's context
//     // and other initialization data.
//     eframe::run_native(
//         "Egui Hello World App", // The title of the window.
//         options, // The native options configured above.
//         Box::new(|_cc| Ok(Box::new(MyApp::default()))), // Initialize our MyApp.
//     )
// }
//
//
fn json_highlighter(ui: &egui::Ui, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();

    let base_format = TextFormat {
        color: ui.visuals().text_color(),
        font_id: TextStyle::Monospace.resolve(ui.style()),
        ..Default::default()
    };

    // Define colors for different JSON elements
    let string_color = Color32::from_rgb(255, 150, 80);    // Orange-ish for strings
    let number_color = Color32::from_rgb(150, 200, 255);   // Light blue for numbers
    let boolean_color = Color32::from_rgb(150, 255, 150);  // Light green for booleans
    let null_color = Color32::from_rgb(255, 150, 255);     // Pink/purple for null

    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    while current_pos < chars.len() {
        let mut appended = false;

        // Try to match specific keywords first
        if text[current_pos..].starts_with("true") {
            job.append("true", 0.0, TextFormat { color: boolean_color, ..base_format.clone() });
            current_pos += 4;
            appended = true;
        } else if text[current_pos..].starts_with("false") {
            job.append("false", 0.0, TextFormat { color: boolean_color, ..base_format.clone() });
            current_pos += 5;
            appended = true;
        } else if text[current_pos..].starts_with("null") {
            job.append("null", 0.0, TextFormat { color: null_color, ..base_format.clone() });
            current_pos += 4;
            appended = true;
        }
        // Handle strings (very basic: assumes no escaped quotes inside)
        else if chars[current_pos] == '"' {
            let start_quote = current_pos;
            current_pos += 1;
            while current_pos < chars.len() && chars[current_pos] != '"' {
                if chars[current_pos] == '\\' && current_pos + 1 < chars.len() {
                    current_pos += 1;
                }
                current_pos += 1;
            }
            if current_pos < chars.len() && chars[current_pos] == '"' {
                current_pos += 1; // Consume closing quote
                let s = &text[start_quote..current_pos];
                job.append(s, 0.0, TextFormat { color: string_color, ..base_format.clone() });
                appended = true;
            } else {
                job.append(&text[start_quote..], 0.0, base_format.clone());
                current_pos = text.len();
                appended = true;
            }
        }
        // Handle numbers
        else if chars[current_pos].is_ascii_digit() || chars[current_pos] == '-' || chars[current_pos] == '+' {
            let start_num = current_pos;
            while current_pos < chars.len() && (chars[current_pos].is_ascii_digit() || chars[current_pos] == '.' || chars[current_pos] == '-' || chars[current_pos] == '+') {
                current_pos += 1;
            }
            let n = &text[start_num..current_pos];
            job.append(n, 0.0, TextFormat { color: number_color, ..base_format.clone() });
            appended = true;
        }

        // If nothing specific matched, append character by character
        if !appended {
            let c = chars[current_pos];
            job.append(&c.to_string(), 0.0, base_format.clone());
            current_pos += 1;
        }
    }
    job
}


fn calculate_desired_rows_from_available_height(ui: &mut Ui, available_height: f32) -> usize {
    // Get the height of a single line of text for the default Body style
    // TextEdit typically uses the Body text style by default.
    let row_height = ui.text_style_height(&TextStyle::Body);

    // Add some padding/spacing, as actual lines might have a bit more vertical space
    // You might need to fine-tune this value based on your theme and desired look.
    let effective_row_height = row_height + ui.spacing().item_spacing.y / 2.0; // Half of item_spacing as a common practice

    // Calculate the number of rows
    if effective_row_height > 0.0 {
        (available_height / effective_row_height).floor() as usize
    } else {
        1 // Prevent division by zero, ensure at least one row
    }
}

// JQ Execution Function
fn execute_jq_query(json_input: &str, query: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::{Command, Stdio}; // Ensure Stdio is in scope

    let mut child = Command::new("jq")
        .arg(query)
        // Corrected lines: Use Stdio::piped() instead of Stdio::Piped
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to spawn jq process. Is jq installed and in PATH? {}",
                e
            )
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(json_input.as_bytes())
            .map_err(|e| format!("Failed to write to jq stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for jq process: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// Define our application struct.
// We use `Default` trait to easily create an instance with default values.
// #[cfg(feature = "egui_example")]
#[derive(Default)]
struct JsonFormatterApp {
    input_json: String,
    // Store the parsed JSON Value directly for structured display
    parsed_json_value: Option<Value>,
    error_message: Option<String>,
    search_query: String,

    // New fields for JQ integration:
    jq_query_input: String,    // The text field for user's JQ query
    cached_layout_job: Option<LayoutJob>,
        last_input_json: String,
}

// Implement the `eframe::App` trait for our `MyApp` struct.
// This trait defines the main lifecycle methods for an egui application.
// #[cfg(feature = "egui_example")]
impl eframe::App for JsonFormatterApp {
    // The `update` method is called repeatedly to redraw the GUI.
    // This is where you define your UI elements.
    //
    // `ctx`: The egui Context, which holds the state of the GUI.
    // `_frame`: The eframe Frame, used for interacting with the native window (e.g., requesting repaint).
    // `ui`: The egui Ui, which represents the current region where widgets can be added.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Create a central panel that takes up all available space.
        // Most egui applications will have a central panel or a top/side panel.
        //
        // egui::TopBottomPanel::top("header_panel").show(ctx, |ui| {
        //     ui.label("This is the header (full width)");
        // });
        // egui::TopBottomPanel::bottom("footer_panel")
        //     .exact_height(0.0) // No explicit height, but still full width
        //     .show(ctx, |ui| {
        //         // If you remove the heading, it will truly be 0 height visually
        //         // ui.heading("bottom"); // This will make it visible, proving it's full width
        //         ui.horizontal(|ui| {
        //             ui.add_space(50.0); // Add some space to show it spans
        //             ui.label(
        //                 "This is the footer panel content, even with 0 height, it's full width.",
        //             );
        //         });
        //     });
        // Central panel fills the remaining space
        egui::CentralPanel::default()

            .show(ctx, |ui| {
 ui.horizontal(|ui| {
                if ui.button("Format JSON").clicked() {
                    self.error_message = None; // Clear previous errors
                    self.parsed_json_value = None; // Clear previous parsed value

                    match parse_json_to_value(&self.input_json) {
                        Ok(value) => {

                            // self.parsed_json_value = Some(value);
                            match serde_json::to_string_pretty(&value) {
                                                                Ok(pretty_json_string) => {
                                                                    self.input_json = pretty_json_string; // Update the input area
                                                                    self.parsed_json_value = Some(value); // Keep the parsed value for the collapsible view
                                                                }
                                                                Err(e) => {
                                                                    self.error_message = Some(format!("Error pretty-printing JSON: {}", e));
                                                                }
                                                            }
                        }
                        Err(e) => {
                            self.error_message = Some(e);
                        }
                    }
                }

                // Add the "Copy to Clipboard" button
                                   if let Some(value) = &self.parsed_json_value {
                                       if ui.button("Copy Formatted JSON").clicked() {
                                           match serde_json::to_string_pretty(value) {
                                               Ok(pretty_json) => {
                                                   ctx.copy_text(pretty_json);
                                               }
                                               Err(e) => {
                                                   self.error_message = Some(format!("Error serializing JSON: {}", e));
                                               }
                                           }
                                       }
                                   }
                               });
                ui.columns(2, |columns| {
                    // Column 1
                    columns[0].vertical(|ui| {
                        ui.set_width(ui.available_width()); // Ensure it uses its full allocated width
                        ui.set_height(ui.available_height());
                        ui.add_space(5.0);

                        ui.label("Paste your JSON here:");
                        ui.add_space(5.0);
                        let available_height = ui.available_height(); // Or ui.available_rect_before_wrap().height()

                                                         // Calculate the desired number of rows
                        let calculated_rows = calculate_desired_rows_from_available_height(ui, available_height);

                        let mut layouter = {
                                        // We need to capture `self` (specifically, its mutable parts) here.
                                        // We are creating a closure that can mutate `self.cached_layout_job`
                                        // and `self.last_input_json`, and read `self.input_json`.
                                        // The `move` keyword ensures that `layouter` takes ownership of these values,
                                        // but since we're passing `&mut self` into `update`, we can't `move self` directly.
                                        // Instead, we capture mutable references to the fields needed by the layouter.
                                        // This means the lifetime of `layouter` is tied to the lifetime of `&mut self`.

                                        // To solve the borrow conflict:
                                        // We pass the string to the layouter (which is `TextEdit`'s content)
                                        // and we also need access to the cache, which belongs to `self`.
                                        // The TextEdit::layouter expects FnMut, so the layouter can modify its captured state.

                                        // The fix is to make sure the parts of self that layouter needs
                                        // are distinct from the parts TextEdit needs for its value (&mut self.input_json).
                                        //
                                        // This pattern is tricky because TextEdit wants a FnMut.
                                        // A common pattern is to make the layouter a method on `self` or a function
                                        // that takes the required mutable references.

                                        // Let's create a *helper function* for the layouter logic
                                        // to avoid complex lifetime issues with nested closures capturing self.
                                        // This function will take the mutable references it needs from `self`.
                                        let input_json_ref = &self.input_json; // Immutable borrow for layouter to read
                                        let last_input_json_ref = &mut self.last_input_json; // Mutable borrow for layouter to update
                                        let cached_layout_job_ref = &mut self.cached_layout_job; // Mutable borrow for layouter to update

                                        move |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                            // Check if input_json has changed relative to our last_input_json for caching
                                            if string != *last_input_json_ref || cached_layout_job_ref.is_none() {
                                                let new_job = json_highlighter(ui, string);
                                                *cached_layout_job_ref = Some(new_job);
                                                *last_input_json_ref = string.to_string(); // Update last seen string for the cache
                                            }
                                            let mut layout_job = cached_layout_job_ref.clone().unwrap_or_default();
                                            layout_job.wrap.max_width = wrap_width;
                                            ui.fonts(|f| f.layout_job(layout_job))
                                        }
                                    };

                        egui::ScrollArea::vertical().id_salt("raw_json_scroll_area_v").show(ui, |ui| {

                            egui::ScrollArea::horizontal().id_salt("raw_json_scroll_area_h").show(ui, |horizontal_ui| {



                        horizontal_ui.add(
                            // egui::TextEdit::multiline(&mut self.input_json)
                            //     .desired_width(f32::INFINITY)
                            //     // .desired_rows(100) // Example: set initial rows for height
                            //     .background_color(Color32::TRANSPARENT)
                            //     .frame(true)
                            //     .desired_rows(calculated_rows.max(50)),
                            egui::TextEdit::multiline(&mut self.input_json)
                                                    .desired_width(f32::INFINITY)
                                                    .background_color(Color32::from_rgb(40,40,40))
                                                    .frame(true)
                                                    .desired_rows(calculated_rows.max(50))
                                                    .layouter(&mut layouter)

                        );
                            });
                        });

                        ui.add_space(ui.available_height());
                    });


                    columns[1].vertical(|ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| { // New: Horizontal layout for search input and button
                                                   ui.label("Search:");
                                                   ui.text_edit_singleline(&mut self.search_query);
                                                   // No explicit "Search" button needed, as typing updates search_query
                                                   // and the view will re-render automatically.
                                                   if ui.button("Clear Search").clicked() {
                                                       self.search_query.clear();
                                                   }
                                               });

                        // JQ Section
                                            // ui.heading("JQ Query");
                                            ui.horizontal(|ui| {
                                                ui.label("JQ Filter:");
                                                ui.text_edit_singleline(&mut self.jq_query_input);
                                                    // .hint_text(".data[0].event_dates_id");
                                                if ui.button("Run").clicked() {
                                                    // self.jq_output = None;
                                                    // self.jq_error = None;

                                                    if self.input_json.is_empty() {
                                                        self.error_message = Some("No JSON input provided to run JQ against.".to_string());
                                                    } else if self.jq_query_input.is_empty() {
                                                        self.error_message = Some("JQ query field cannot be empty.".to_string());
                                                    } else {
                                                        match execute_jq_query(&self.input_json, &self.jq_query_input) {
                                                            Ok(output) => {
                                                                // self.jq_output = Some(output);

                                                                match parse_json_to_value(&output) {
                                                                                        Ok(value) => {

                                                                                            // self.parsed_json_value = Some(value);
                                                                                            match serde_json::to_string_pretty(&value) {
                                                                                                Ok(_pretty_json_string) => {
                                                                                                // self.input_json = pretty_json_string; // Update the input area
                                                                                                self.parsed_json_value = Some(value); // Keep the parsed value for the collapsible view
                                                                                                }
                                                                                                Err(e) => {
                                                                                                self.error_message = Some(format!("Error pretty-printing JSON: {}", e));
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                        Err(e) => {
                                                                                            self.error_message = Some(e);
                                                                                        }
                                                                                    }

                                                            }
                                                            Err(e) => {
                                                                // self.jq_error = Some(e);
                                                                 self.error_message = Some(format!("Error : {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                                if ui.button("Clear").clicked() {
                                                    self.jq_query_input.clear();
                                                    match parse_json_to_value(&self.input_json) {
                                                                            Ok(value) => {
                                                                                // self.parsed_json_value = Some(value);
                                                                                match serde_json::to_string_pretty(&value) {
                                                                                    Ok(_pretty_json_string) => {
                                                                                        // self.input_json = pretty_json_string; // Update the input area
                                                                                        self.parsed_json_value = Some(value); // Keep the parsed value for the collapsible view
                                                                                    }
                                                                                    Err(e) => {
                                                                                        self.error_message = Some(format!("Error pretty-printing JSON: {}", e));
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(e) => {
                                                                                self.error_message = Some(e);
                                                                            }
                                                                        }
                                                    // self.jq_output = None;
                                                    // self.jq_error = None;
                                                }
                                            });
                                             // Render the parsed JSON value if available
                                             if let Some(value) = &self.parsed_json_value {
                                                 egui::ScrollArea::vertical().id_salt("formatted_json_scroll_area_v").show(ui, |ui| {
                                                      egui::ScrollArea::horizontal().id_salt("formatted_json_scroll_area_h").show(ui, |horizontal_ui| {
                                                     render_json_value(horizontal_ui, None, value, "$",&self.search_query);
                                                      });
                                                 });
                                             } else {
                                                 ui.label("Enter JSON above and click 'Format JSON' to see the collapsible structure.");
                                             }
                                             ui.separator(); // Visual separator


                                                                 // Display JQ Output or Error in a CollapsingHeader
                                                                 // if self.jq_output.is_some() || self.jq_error.is_some() {
                                                                 //     CollapsingHeader::new(RichText::new("JQ Result").strong().color(Color32::WHITE))
                                                                 //         .id_salt("jq_result_header")
                                                                 //         .default_open(true) // Keep open if there's a result
                                                                 //         .show(ui, |ui| {
                                                                 //             ui.indent("jq_result_indent", |ui| {
                                                                 //                 if let Some(output) = &self.jq_output {
                                                                 //                     egui::ScrollArea::vertical().id_salt("jq_output_scroll_v").show(ui, |ui| {
                                                                 //                         egui::ScrollArea::horizontal().id_salt("jq_output_scroll_h").show(ui, |ui| {
                                                                 //                             // Display JQ output in a read-only multiline text edit
                                                                 //                             ui.add(egui::TextEdit::multiline(&mut output.clone())
                                                                 //                                 .desired_width(ui.available_width())
                                                                 //                                 .desired_rows(10) // Give it some initial height
                                                                 //                                 .font(TextStyle::Monospace)); // Use monospace for code/JSON
                                                                 //                         });
                                                                 //                     });
                                                                 //                 } else if let Some(error) = &self.jq_error {
                                                                 //                     ui.colored_label(Color32::RED, format!("Error: {}", error));
                                                                 //                 }
                                                                 //             });
                                                                 //         });
                                                                 // }

                                                                 // ui.add_space(ui.available_height());


                    });
                });
                ui.add_space(ui.available_height());
            });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "JSON Pretty Formatter",
        native_options,
        Box::new(|_cc| Ok(Box::<JsonFormatterApp>::default())),
    )
}
