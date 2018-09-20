extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mustache;
extern crate pulldown_cmark;
extern crate syntect;
extern crate toml;

mod markdown_renderer;

use std::{env, io, fs};
use std::path::{Path, PathBuf};
use mustache::MapBuilder;
use markdown_renderer::{Renderer};

#[derive(Debug, Deserialize)]
struct Config {
    portrait_path: String,
    google_analytics_tracking_id: String,
}

#[derive(Debug, Deserialize)]
struct PostMetadata {
    title: String,
    date: String,
    format: String,
    publish: Option<bool>,
}

#[derive(Debug, Serialize)]
struct Post {
    title: String,
    date: String,
    url: String,
    dir: PathBuf,
    format: String,
}

#[derive(Debug, Deserialize)]
struct ProjectMetadata {
    title: String,
    filename: String,
    format: String,
    publish: Option<bool>,
    js_file: Option<String>,
}

#[derive(Debug, Serialize)]
struct Project {
    title: String,
    filename: String,
    format: String,
    url: String,
    dir: PathBuf,
    js_file: Option<String>,
}

fn main() -> io::Result<()> {
    
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} CONTENT_DIR OUT_DIR", args[0]);
        return Ok(());
    }

    // Root is for files owned by the generator (ie templates, etc).
    let root_dir = Path::new("src");

    let content_dir = Path::new(&args[1]);

    let out_dir = Path::new(&args[2]);

    let config_text = fs::read_to_string(content_dir.join("config.toml"))?;
    let config: Config = toml::from_str(config_text.as_str()).unwrap();

    fs::remove_dir_all(out_dir)?;
    fs::create_dir(out_dir)?;

    // js
    fs::copy(content_dir.join("bundle.js"), out_dir.join("bundle.js"))?;

    // styles
    fs::copy(root_dir.join("styles.css"), out_dir.join("styles.css"))?;

    // portrait image
    let mut portrait_url = Path::new(&config.portrait_path).to_path_buf();
    let filename = portrait_url.file_name().unwrap().to_os_string();
    portrait_url.pop();
    let portrait_dir = out_dir.join(portrait_url);
    fs::create_dir_all(&portrait_dir)?;
    fs::copy(content_dir.join(&config.portrait_path), portrait_dir.join(filename))?;

    // index

    let ga_template = mustache::compile_path(
        root_dir.join("google_analytics.mustache"))
        .expect("Failed to compile ga template");
    let ga_data = MapBuilder::new()
       .insert_str("google_analytics_tracking_id",
                   config.google_analytics_tracking_id)
       .build();
    let ga_string = ga_template.render_data_to_string(&ga_data)
        .unwrap();

    let index_template = mustache::compile_path(root_dir.join("index.mustache"))
        .expect("Failed to compile index template");
    let index_data = MapBuilder::new()
       .insert_str("portrait_url", config.portrait_path)
       // TODO: this is a hack to workaround the fact that rust-mustache
       // doesn't seem to be passing the context down to nested partials
       .insert_str("ga_partial", ga_string.clone())
       .build();

    let index_string = index_template.render_data_to_string(&index_data)
        .unwrap();

    fs::write(out_dir.join("index.html"), index_string)?;

    make_blog_page(&root_dir, &out_dir, &content_dir, &ga_string)?;
    make_projects_page(&root_dir, &out_dir, &content_dir, &ga_string)?;

    // apps are expected to provide their own entry point. This simply
    // creates a place for them to live.
    let app_out_dir = out_dir.join("apps");
    fs::create_dir(app_out_dir)?;

    Ok(())
}

fn make_blog_page(
    root_dir: &Path,
    out_dir: &Path,
    content_dir: &Path,
    ga_string: &String) -> io::Result<()> {
    
    let blog_root_dir = root_dir.join("blog");

    let blog_out_dir = out_dir.join("blog");
    let posts_out_dir = blog_out_dir.clone();

    let blog_content_dir = content_dir.join("blog");

    fs::create_dir(&blog_out_dir)?;
    //fs::create_dir(&posts_out_dir)?;
   
    let blog_template = mustache::compile_path(
        blog_root_dir.join("index.mustache"))
        .expect("Failed to compile blog template");

    let mut post_dirs = Vec::new();
    for entry in fs::read_dir(blog_content_dir.join("posts"))? {
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
        let post_md: PostMetadata = toml::from_str(metadata.as_str()).unwrap();

        // skip any posts with false publish metadata
        if let Some(publish) = post_md.publish {
            if !publish {
                continue;
            }
        }

        //let url = Path::new("posts").join(post_dir.file_name().unwrap());
        let url = Path::new("/blog").join(post_dir.file_name().unwrap());
        let mut url_string = url.into_os_string().into_string().unwrap();
        url_string.push('/');
        //println!("{:?}", url);
        let date = get_date_from_iso8601(&post_md.date);
        posts.push(Post {
            title: post_md.title,
            date: date,
            url: url_string,
            dir: post_dir.clone(),
            format: post_md.format,
        });
    }

    posts.sort_unstable_by(|a, b| a.date.cmp(&b.date));
    posts.reverse();

    render_posts(&root_dir, &posts_out_dir, &posts, &ga_string)?;

    let blog_data = MapBuilder::new()
        .insert("posts", &posts).unwrap()
        // TODO: this is a hack to workaround the fact that rust-mustache
        // doesn't seem to be passing the context down to nested partials
        .insert("ga_partial", &ga_string).unwrap()
        .build();

    //let blog_string = blog_template.render_to_string(&blog_data)
    //    .unwrap();
    let blog_string = blog_template.render_data_to_string(&blog_data)
        .unwrap();

    fs::write(blog_out_dir.join("index.html"), blog_string)?;

    Ok(())
}

fn render_posts(
    content_dir: &Path,
    out_dir: &PathBuf,
    posts: &Vec<Post>,
    ga_string: &String) -> io::Result<()> {

    for post in posts {
        let dir = post.dir.clone();
        let name = dir.file_name().unwrap();
        fs::create_dir(out_dir.join(name))?;

        let src_path = dir.join("post.md");
        let content = fs::read_to_string(src_path)?;

        let html = render_to_html(&content, &post.format);

        let post_template = mustache::compile_path(content_dir.join("post.mustache"))
            .expect("Failed to compile post template");

        let post_data = MapBuilder::new()
           .insert_str("title", post.title.clone())
           .insert_str("date", post.date.clone())
           .insert_str("content", html)
           // TODO: this is a hack to workaround the fact that rust-mustache
           // doesn't seem to be passing the context down to nested partials
           .insert("ga_partial", &ga_string).unwrap()
           .build();

        let post_string = post_template.render_data_to_string(&post_data)
            .unwrap();

        fs::write(out_dir.join(name).join("index.html"), post_string)?;
    }

    Ok(())
}

// TODO: there's a lot of duplicated logic between this and the blog page
// above.
fn make_projects_page(
    root_dir: &Path,
    out_dir: &Path,
    content_dir: &Path,
    ga_string: &String) -> io::Result<()> {

    let projects_root_dir = root_dir.join("projects");
    let projects_out_dir = out_dir.join("projects");
    let projects_content_dir = content_dir.join("projects");

    fs::create_dir(&projects_out_dir)?;
    //fs::create_dir(&posts_out_dir)?;
   
    let projects_template = mustache::compile_path(
        projects_root_dir.join("index.mustache"))
        .expect("Failed to compile projects template");

    let project_dirs = get_dirs(&projects_content_dir)?;

    let mut projects = Vec::new();

    for project_dir in project_dirs.into_iter() {
        let metadata = fs::read_to_string(project_dir.join("metadata.toml"))?;
        let project_md: ProjectMetadata = toml::from_str(metadata.as_str()).unwrap();

        // skip any posts with false publish metadata
        if let Some(publish) = project_md.publish {
            if !publish {
                continue;
            }
        }

        let url = Path::new("/projects").join(project_dir.file_name().unwrap());
        let mut url_string = url.into_os_string().into_string().unwrap();
        url_string.push('/');
        projects.push(Project {
            title: project_md.title,
            filename: project_md.filename,
            format: project_md.format,
            url: url_string,
            dir: project_dir.clone(),
            js_file: project_md.js_file,
        });
    }

    render_projects(&root_dir, &projects_out_dir, &projects, &ga_string)?;

    let projects_data = MapBuilder::new()
        .insert("projects", &projects).unwrap()
        // TODO: this is a hack to workaround the fact that rust-mustache
        // doesn't seem to be passing the context down to nested partials
        .insert("ga_partial", &ga_string).unwrap()
        .build();

    let projects_string = projects_template.render_data_to_string(&projects_data)
        .unwrap();

    fs::write(projects_out_dir.join("index.html"), projects_string)?;

    Ok(())
}

fn render_projects(
    content_dir: &Path,
    out_dir: &PathBuf,
    projects: &Vec<Project>,
    ga_string: &String) -> io::Result<()> {

    for project in projects {
        let dir = project.dir.clone();
        let name = dir.file_name().unwrap();
        fs::create_dir(out_dir.join(name))?;

        let src_path = dir.join(&project.filename);
        let content = fs::read_to_string(src_path)?;

        let html = render_to_html(&content, &project.format);

        let project_template = mustache::compile_path(content_dir.join("project.mustache"))
            .expect("Failed to compile project template");

        let project_data = MapBuilder::new()
           .insert_str("title", project.title.clone())
           .insert_str("content", html)
           // TODO: this is a hack to workaround the fact that rust-mustache
           // doesn't seem to be passing the context down to nested partials
           .insert("ga_partial", &ga_string).unwrap()
           .build();

        let project_string = project_template.render_data_to_string(&project_data)
            .unwrap();

        fs::write(out_dir.join(name).join("index.html"), project_string)?;

        if let Some(js_filename) = &project.js_file {
            println!("{:?}", out_dir.join(js_filename));
            fs::copy(dir.join(js_filename), out_dir.join(name).join(js_filename))?;
        }
    }

    Ok(())
}

fn get_date_from_iso8601(iso8601: &String) -> String {
    iso8601.split("T").collect::<Vec<&str>>()[0].to_string()
}

fn get_dirs(path: &Path) -> io::Result<Vec<PathBuf>> {

    let mut dirs = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }

    Ok(dirs)
}

fn render_to_html(content_string: &String, format: &str) -> String {
    match format {
        "markdown" => {
            let renderer = Renderer::new();
            renderer.render(&content_string).clone()
        }
        "html" => {
            content_string.clone()
        }
        _ => {
            "<h1>Fail - invalid format</h1>".to_string()
        }
    }
}
