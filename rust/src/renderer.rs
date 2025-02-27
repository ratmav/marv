use maud::{html, PreEscaped, DOCTYPE};
use pulldown_cmark::{html, Event, Options, Parser};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

// Convert markdown to HTML
pub fn markdown_to_html(markdown: &str, filename: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    // Custom handling to preserve code blocks
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();

    // Use a custom renderer to handle code blocks properly
    let mut events: Vec<Event> = Vec::new();
    for event in parser {
        events.push(event);
    }

    // Process the events
    html::push_html(&mut html_output, events.into_iter());

    let page = html! {
    (DOCTYPE)
    html {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no";
            title { "marv" }
            style {
                "body {
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
                        line-height: 1.6;
                        max-width: 800px;
                        margin: 0 auto;
                        padding: 2rem;
                        background-color: #1e1e1e;
                        color: #e0e0e0;
                    }
                    pre, code {
                        background-color: #2d2d2d;
                        border-radius: 3px;
                        padding: 0.2em 0.4em;
                        font-family: monospace;
                        color: #d4d4d4;
                    }
                    pre code {
                        padding: 0;
                    }
                    pre.syntax-highlight {
                        padding: 1em;
                        overflow: auto;
                    }
                    /* Syntax highlighting colors */
                    .syntax-highlight .comment { color: #65737e; }
                    .syntax-highlight .string { color: #a3be8c; }
                    .syntax-highlight .keyword { color: #b48ead; }
                    .syntax-highlight .function { color: #8fa1b3; }
                    .syntax-highlight .number { color: #d08770; }
                    .syntax-highlight .constant { color: #d08770; }
                    .syntax-highlight .type { color: #ebcb8b; }
                    .syntax-highlight .tag { color: #bf616a; }
                    .syntax-highlight .attribute { color: #d08770; }
                    .syntax-highlight .title { color: #8fa1b3; font-weight: bold; }
                    .syntax-highlight .attribute-value { color: #a3be8c; }
                    table {
                        border-collapse: collapse;
                        width: 100%;
                    }
                    table, th, td {
                        border: 1px solid #444;
                    }
                    th, td {
                        padding: 8px;
                    }
                    tr:nth-child(even) {
                        background-color: #2a2a2a;
                    }
                    img {
                        max-width: 100%;
                    }
                    blockquote {
                        border-left: 4px solid #444;
                        padding-left: 1rem;
                        margin-left: 0;
                        color: #aaa;
                    }
                    a {
                        color: #58a6ff;
                    }
                    h1, h2, h3, h4, h5, h6 {
                        margin-top: 1.5em;
                        margin-bottom: 0.5em;
                    }
                    p, ul, ol {
                        margin: 1em 0;
                    }
                    /* File path header styling */
                    #file-path {
                        margin-bottom: 2em;
                    }
                    #file-path h1 {
                        font-size: 1.2em;
                        margin-top: 0.5em;
                        margin-bottom: 0.5em;
                        word-break: break-all;
                        overflow-wrap: break-word;
                    }
                    .update-indicator {
                        position: fixed;
                        bottom: 10px;
                        right: 10px;
                        background-color: #2e6930;
                        color: #e0e0e0;
                        padding: 5px 10px;
                        border-radius: 5px;
                        font-size: 12px;
                        opacity: 0;
                        transition: opacity 0.3s;
                    }
                    .visible {
                        opacity: 1;
                    }"
            }
            // Mermaid script
            script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js" {}
            script {
                "document.addEventListener('DOMContentLoaded', function() {
                        mermaid.initialize({
                            startOnLoad: true,
                            theme: 'dark'
                        });
                    });"
            }
        }
        body {
            // Add filepath header
            header id="file-path" {
                h1 { (filename) }
                hr;
            }

            div id="content" {
                (PreEscaped(process_mermaid_diagrams(&html_output)))
            }

            /*
            script {
                "
                // Initialize Mermaid diagrams when page loads
                document.addEventListener('DOMContentLoaded', function() {
                    if (window.mermaid) {
                        window.mermaid.init(undefined, document.querySelectorAll('.mermaid'));
                    }

                    window.lastReloadTime = Date.now();
                });

                // Simple polling to reload the page
                setInterval(function() {
                    // Only reload if it's been at least 2 seconds since last reload
                    // This prevents excessive reloading when the server is slow
                    /*
                    if (Date.now() - window.lastReloadTime > 2000) {
                        // Update timestamp before reload
                        window.lastReloadTime = Date.now();
                        // Perform the reload
                        window.location.reload();
                    }
                    */
                    window.location.reload();
                }, 1000);
                "
                }
            */
            }
        }
    };

    page.into_string()
}

// Process markdown to detect and prepare Mermaid diagrams and syntax highlighting
pub fn process_mermaid_diagrams(html: &str) -> String {
    // Set up syntax highlighting
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    // First pass: extract code blocks for syntax highlighting
    let mut intermediate = html.to_string();

    // A more flexible pattern for code blocks that handles entities better
    let re_code =
        regex::Regex::new(r#"<pre><code class="language-([^"]+)">([\s\S]*?)</code></pre>"#)
            .unwrap();
    for cap in re_code.captures_iter(html) {
        let lang = cap.get(1).unwrap().as_str();
        let code = cap.get(2).unwrap().as_str();

        // Skip mermaid diagrams - they need special handling
        if lang == "mermaid" {
            continue;
        }

        // Unescape HTML entities (like &quot; back to ")
        let unescaped_code = html_escape::decode_html_entities(code).to_string();

        // Try to find the syntax for this language
        if let Some(syntax) = ss.find_syntax_by_token(lang) {
            // Generate highlighted HTML and handle any errors
            match highlighted_html_for_string(&unescaped_code, &ss, syntax, theme) {
                Ok(highlighted) => {
                    // Create replacement with pre-highlighted code
                    let replacement = format!(
                        r#"<pre class="syntax-highlight language-{}">{}</pre>"#,
                        lang, highlighted
                    );

                    // Replace in the intermediate HTML
                    let full_match = cap.get(0).unwrap().as_str();
                    intermediate = intermediate.replace(full_match, &replacement);
                }
                Err(_) => {
                    // If highlighting fails, keep the original code block
                    continue;
                }
            }
        }
    }

    // Second pass: handle mermaid diagrams
    let intermediate = intermediate
        .replace(
            "<pre><code class=\"language-mermaid\">",
            "<div class=\"mermaid\">",
        )
        .replace("<pre><code>mermaid", "<div class=\"mermaid\">");

    // Find all mermaid div openings and only replace their corresponding closing tags
    let mut result = String::new();
    let mut in_mermaid = false;

    for line in intermediate.lines() {
        if line.contains("<div class=\"mermaid\">") {
            in_mermaid = true;
            result.push_str(line);
            result.push('\n');
        } else if in_mermaid && line.contains("</code></pre>") {
            in_mermaid = false;
            // Replace closing tags and preserve newline
            result.push_str(&line.replace("</code></pre>", "</div>"));
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}
