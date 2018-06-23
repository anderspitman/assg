use std::borrow::Cow::Owned;
use pulldown_cmark::{Parser, Event, Tag, html};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet};
use syntect::html::highlighted_snippet_for_string;
use std::collections::HashMap;

pub struct Renderer {
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
        }
    }

    pub fn render(self, markdown_text: &String) -> String {

        let lang_map = build_language_map();

        let ss = SyntaxSet::load_defaults_nonewlines();

        //for syntax in ss.syntaxes() {
        //    println!("{:?}", syntax.name);
        //}

        let ts = ThemeSet::load_defaults();
        //let theme = &ts.themes["base16-ocean.dark"];
        let theme = &ts.themes["base16-eighties.dark"];
        //let theme = &ts.themes["Solarized (light)"];

        let mut in_code_block = false;
        let mut code = String::new();
        let mut syntax_name = lang_map.get("bash").unwrap();

        let parser = Parser::new(&markdown_text).map(|event| {
            //println!("{:?}", event);

            match event {
                Event::Start(Tag::CodeBlock(language)) => {
                    in_code_block = true;
                    syntax_name = lang_map.get(&language.to_string())
                        .expect(&format!("{:?} not in language map", language));
                    Event::Html(Owned("<div class='code'>".to_string()))
                },
                Event::End(Tag::CodeBlock(_)) => {
                    in_code_block = false;

                    let syntax = ss.find_syntax_by_name(
                        syntax_name.as_str()).unwrap();

                    let mut html = highlighted_snippet_for_string(
                        &code.to_string(), syntax, theme);

                    html.push_str("</div>");

                    code = String::new();
                    Event::Html(Owned(html))
                },
                Event::Text(text) => {

                    if in_code_block {
                        code += &text.to_string();
                        Event::Text(Owned("".to_string()))
                    }
                    else {
                        Event::Text(text)
                    }
                }
                _ => event
            }
        });

        let mut html = String::new();

        html::push_html(&mut html, parser);

        html
    }
}

fn build_language_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("".to_string(), "Plain Text".to_string());
    map.insert("bash".to_string(), "Shell-Unix-Generic".to_string());
    map.insert("javascript".to_string(), "JavaScript".to_string());
    map.insert("html".to_string(), "HTML".to_string());
    map.insert("json".to_string(), "JSON".to_string());
    map.insert("toml".to_string(), "Plain Text".to_string());
    map.insert("rust".to_string(), "Rust".to_string());
    map.insert("css".to_string(), "CSS".to_string());
    map.insert("python".to_string(), "Python".to_string());
    map
}
