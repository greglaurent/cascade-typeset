use askama::Template;

// Define the template struct with the escape override
#[derive(Template)]
#[template(path = "css/example.css", escape = "none")]
struct CssTemplate<'a> {
    primary_color: &'a str,
    secondary_color: &'a str,
    radius_px: u32,
    font_stack: &'a str,
}

fn main() {
    // Instantiate your CSS variables
    let stylesheet = CssTemplate {
        primary_color: "#1a1a1a",
        secondary_color: "#ff4500",
        radius_px: 8,
        font_stack: "'Helvetica Neue', Arial, sans-serif",
    };

    // Render the template to a string
    match stylesheet.render() {
        Ok(css_output) => {
            println!("{}", css_output);
            
            // Optional: Save it to a file
            // std::fs::write("generated.css", css_output).expect("Unable to write file");
        }
        Err(err) => eprintln!("Failed to render CSS template: {}", err),
    }
}
