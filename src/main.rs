extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mustache;
extern crate pulldown_cmark;
extern crate syntect;
extern crate toml;

mod markdown_renderer;

use std::{io, fs};
use std::path::{Path, PathBuf};
use mustache::MapBuilder;
use std::collections::HashMap;
use markdown_renderer::{Renderer};


#[derive(Debug, Deserialize)]
struct PostConfig {
    title: String,
    date: String,
    format: String,
}

#[derive(Debug, Serialize)]
struct Post {
    title: String,
    date: String,
    url: String,
    dir: PathBuf,
}

fn main() -> io::Result<()> {
    
    let src_dir = Path::new("src");
    let out_dir = Path::new("dist");
    let blog_out_dir = out_dir.join("blog");
    let posts_out_dir = blog_out_dir.clone();

    fs::remove_dir_all(out_dir)?;
    fs::create_dir(out_dir)?;

    // styles
    
    fs::copy(src_dir.join("styles.css"), out_dir.join("styles.css"))?;

    // index

    let index_template = mustache::compile_path(src_dir.join("index.mustache"))
        .expect("Failed to compile index template");

    let index_data = MapBuilder::new()
       .build();

    let index_string = index_template.render_data_to_string(&index_data)
        .unwrap();

    fs::write(out_dir.join("index.html"), index_string)?;


    // blog
    
    fs::create_dir(&blog_out_dir)?;
    //fs::create_dir(&posts_out_dir)?;
   
    let blog_template = mustache::compile_path("./src/blog/index.mustache")
        .expect("Failed to compile blog template");

    let mut post_dirs = Vec::new();
    for entry in fs::read_dir("src/blog/posts")? {
        let entry = entry?;
        //println!("{:?}", entry.path());
        let path = entry.path();
        if path.is_dir() {
            post_dirs.push(path);
        }
    }

    let mut posts = Vec::new();

    // blog index
    for post_dir in post_dirs.into_iter() {
        let metadata = fs::read_to_string(post_dir.join("metadata.toml"))?;
        let post: PostConfig = toml::from_str(metadata.as_str()).unwrap();
        //let url = Path::new("posts").join(post_dir.file_name().unwrap());
        let url = post_dir.file_name().unwrap();
        //println!("{:?}", url);
        posts.push(Post {
            title: post.title,
            date: post.date,
            url: url.to_os_string().into_string().unwrap(),
            dir: post_dir.clone(),
        });
    }

    render_posts(&posts_out_dir, &posts)?;

    let mut blog_data = HashMap::new();
    blog_data.insert("posts", posts);

    let blog_string = blog_template.render_to_string(&blog_data)
        .unwrap();

    fs::write("dist/blog/index.html", blog_string)?;

    Ok(())
}

fn render_posts(out_dir: &PathBuf, posts: &Vec<Post>) -> io::Result<()>{

    for post in posts {
        let dir = post.dir.clone();
        let name = dir.file_name().unwrap();
        fs::create_dir(out_dir.join(name))?;

        let src_path = dir.join("post.md");
        let md = fs::read_to_string(src_path)?;

        let renderer = Renderer::new();
        let html = renderer.render(&md);

        let post_template = mustache::compile_path("./src/post.mustache")
            .expect("Failed to compile post template");

        let post_data = MapBuilder::new()
           .insert_str("title", post.title.clone())
           .insert_str("date", post.date.clone())
           .insert_str("content", html)
           .build();

        let post_string = post_template.render_data_to_string(&post_data)
            .unwrap();

        fs::write(out_dir.join(name).join("index.html"), post_string)?;
    }

    Ok(())
}
