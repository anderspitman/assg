extern crate mustache;

use std::{io, fs};
use mustache::MapBuilder;

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

    let blog_data = MapBuilder::new()
       //.insert_str("name", "Venus")
       .build();

    let blog_string = blog_template.render_data_to_string(&blog_data)
        .unwrap();

    fs::write("dist/blog/index.html", blog_string)?;

    Ok(())
}
