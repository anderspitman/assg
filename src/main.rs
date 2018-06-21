extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mustache;

use std::{io, fs};
use mustache::MapBuilder;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct Post {
    name: String
}

fn main() -> io::Result<()> {

    fs::remove_dir_all("dist")?;
    fs::create_dir("dist")?;

    // styles
    
    fs::copy("src/styles.css", "dist/styles.css")?;

    // index

    let index_template = mustache::compile_path("./src/index.mustache")
        .expect("Failed to compile index template");

    let index_data = MapBuilder::new()
       //.insert_str("name", "Venus")
       .build();

    let index_string = index_template.render_data_to_string(&index_data)
        .unwrap();

    fs::write("dist/index.html", index_string)?;


    // blog
    
    fs::create_dir("dist/blog")?;
   
    let blog_template = mustache::compile_path("./src/blog/index.mustache")
        .expect("Failed to compile blog template");

    let mut filenames = Vec::new();
    for file in fs::read_dir("src/blog/posts")? {
        let file = file?;
        filenames.push(file.file_name());
    }

    let mut posts = Vec::new();

    for filename in filenames {
        posts.push(Post { name: filename.into_string().unwrap() });
    }

    let mut blog_data = HashMap::new();
    blog_data.insert("posts", posts);

    let blog_string = blog_template.render_to_string(&blog_data)
        .unwrap();

    fs::write("dist/blog/index.html", blog_string)?;

    Ok(())
}
