use crate::StoryLine::*;
use askama::Template; // bring trait in scope
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::result::Result as StdResult;
#[derive(Deserialize, Debug, PartialEq)]
enum EntryType {
    ACTIVITY,
    #[allow(non_camel_case_types)]
    MINI_ACTIVITY,
    MAINLINE,
    NONE,
}
impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryType::ACTIVITY => write!(f, "activity"),
            EntryType::MINI_ACTIVITY => write!(f, "mini_activity"),
            EntryType::MAINLINE => write!(f, "mainline"),
            EntryType::NONE => write!(f, "none"),
        }
    }
}
#[derive(Deserialize, Debug)]
struct StoryReview {
    id: String,
    name: String,
    #[serde(rename(deserialize = "entryType"))]
    entry_type: String,
    #[serde(rename(deserialize = "storyEntryPicId"))]
    story_entry_pic_id: Option<String>,
    #[serde(rename(deserialize = "infoUnlockDatas"))]
    stories: Vec<Story>,
}
#[derive(Deserialize, Debug, Default)]
struct Story {
    #[serde(rename(deserialize = "storyId"))]
    story_id: String,
    #[serde(rename(deserialize = "storyCode"))]
    story_code: Option<String>,
    #[serde(rename(deserialize = "storyTxt"))]
    story_txt: String,
    #[serde(skip)]
    story_detail: StoryDetail,
}
#[derive(Deserialize, Debug)]
struct StoryLineBase {
    id: i32,
    prop: String,
    attributes: Value,
    #[serde(rename(deserialize = "targetLine"))]
    target_lines: Option<Value>,
    #[serde(rename(deserialize = "endOfOpt"))]
    end_of_opt: Option<bool>,
}
#[derive(Deserialize, Debug, Default)]
struct StoryDetail {
    #[serde(rename(deserialize = "storyCode"))]
    story_code: Option<String>,
    #[serde(rename(deserialize = "avgTag"))]
    avg_tag: String,
    #[serde(rename(deserialize = "storyName"))]
    story_name: String,
    #[serde(rename(deserialize = "storyInfo"))]
    story_info: String,
    #[serde(rename(deserialize = "storyList"))]
    story_lines: Vec<StoryLine>,
}
#[derive(Debug)]
enum StoryLine {
    Dialogue {
        id: i32,
        attributes: DialogueAttributes,
    },
    Text {
        id: i32,
        attributes: TextAttributes,
    },
    Background {
        id: i32,
        attributes: ImageAttributes,
    },
    Image {
        id: i32,
        attributes: ImageAttributes,
    },
    Decision {
        id: i32,
        attributes: DecisionAttributes,
    },
    Predicate {
        id: i32,
        attributes: PredicateAttributes,
        end_of_opt: bool,
    },
    Other {
        id: i32,
    },
}
impl<'de> Deserialize<'de> for StoryLine {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let line_base = StoryLineBase::deserialize(deserializer)?;
        let line = match line_base.prop.as_str() {
            "Image" => StoryLine::Image {
                id: line_base.id,
                attributes: ImageAttributes::deserialize(line_base.attributes)
                    .map_err(serde::de::Error::custom)?,
            },
            "Background" => StoryLine::Background {
                id: line_base.id,
                attributes: ImageAttributes::deserialize(line_base.attributes)
                    .map_err(serde::de::Error::custom)?,
            },
            "name" => StoryLine::Dialogue {
                id: line_base.id,
                attributes: DialogueAttributes::deserialize(line_base.attributes)
                    .map_err(serde::de::Error::custom)?,
            },
            "Sticker" => StoryLine::Text {
                id: line_base.id,
                attributes: TextAttributes::deserialize(line_base.attributes)
                    .map_err(serde::de::Error::custom)?,
            },
            "Subtitle" => StoryLine::Text {
                id: line_base.id,
                attributes: TextAttributes::deserialize(line_base.attributes)
                    .map_err(serde::de::Error::custom)?,
            },
            "Decision" => {
                let tll = line_base.target_lines.unwrap();
                let tl = tll.as_object().unwrap();
                let mut tls: Vec<String> = Vec::new();
                for (_, li) in tl.iter() {
                    tls.push(String::deserialize(li).map_err(serde::de::Error::custom)?);
                }
                let opt = DecisionAttributesPre::deserialize(line_base.attributes)
                    .map_err(serde::de::Error::custom)?;
                let attr = DecisionAttributes {
                    options: opt
                        .options
                        .split(";")
                        .map(|x| x.to_string())
                        .zip(tls.into_iter())
                        .collect(),
                };
                StoryLine::Decision {
                    id: line_base.id,
                    attributes: attr,
                }
            }
            "Predicate" => StoryLine::Predicate {
                id: line_base.id,
                attributes: PredicateAttributes::deserialize(line_base.attributes)
                    .map_err(serde::de::Error::custom)?,
                end_of_opt: line_base.end_of_opt.unwrap_or(false),
            },
            _ => StoryLine::Other { id: line_base.id },
        };
        Ok(line)
    }
}
#[derive(Deserialize, Debug)]
struct ImageAttributes {
    image: Option<String>,
}
#[derive(Deserialize, Debug)]
struct PredicateAttributes {
    references: String,
}
#[derive(Deserialize, Debug)]
struct DecisionAttributesPre {
    options: String,
}
#[derive(Deserialize, Debug)]
struct DecisionAttributes {
    options: Vec<(String, String)>,
}
#[derive(Deserialize, Debug)]
struct DialogueAttributes {
    content: String,
    name: Option<String>,
}
#[derive(Deserialize, Debug)]
struct TextAttributes {
    text: Option<String>,
}

#[derive(Template)] // this will generate the code...
#[template(path = "story.jinja", escape = "none")] // using the template in this path, relative
                                                   // to the `templates` dir in the crate root
struct StoryTemplate<'a> {
    // the name of the struct can be anything
    story: &'a Story, // the field type should match the type, // the field name should match the variable name
    stories: &'a Vec<Story>,
    activities: &'a Vec<&'a StoryReview>,
    activity: &'a StoryReview,
    category: &'a String,
    categories: &'a Vec<String>,
    prev: Option<&'a Story>,
    next: Option<&'a Story>,
    // in your template
}
#[derive(Template)] // this will generate the code...
#[template(path = "activity.jinja", escape = "none")] // using the template in this path, relative
                                                      // to the `templates` dir in the crate root
struct ActivityTemplate<'a> {
    activities: &'a Vec<&'a StoryReview>,
    activity: &'a StoryReview,
    category: &'a String,
    categories: &'a Vec<String>,
    // in your template
}
#[derive(Template)] // this will generate the code...
#[template(path = "category.jinja", escape = "none")] // using the template in this path, relative
                                                      // to the `templates` dir in the crate root
struct CategoryTemplate<'a> {
    activities: &'a Vec<&'a StoryReview>,
    category: &'a String,
    categories: &'a Vec<String>,
    // in your template
}
#[derive(Template)] // this will generate the code...
#[template(path = "index.jinja", escape = "none")] // using the template in this path, relative
                                                   // to the `templates` dir in the crate root
struct IndexTemplate<'a> {
    categories: &'a Vec<String>,
}
fn read_story_reviews() -> StdResult<HashMap<String, Vec<StoryReview>>, Box<dyn Error>> {
    let data = fs::read_to_string("json/zh_CN/story_review_table.json")?;
    let story_reviews_unparsed: Value = serde_json::from_str(&data)?;
    let mut story_reviews: HashMap<String, Vec<StoryReview>> = HashMap::new();
    for (_, item) in story_reviews_unparsed.as_object().unwrap() {
        let story_review: StoryReview = serde_json::from_value(item.clone())?;
        println!("{}", story_review.name);
        if story_reviews.contains_key(&story_review.entry_type) {
            story_reviews
                .get_mut(&story_review.entry_type)
                .unwrap()
                .push(story_review);
        } else {
            story_reviews.insert(story_review.entry_type.clone(), vec![story_review]);
        }
    }
    // println!("{:?}",story_reviews);
    Ok(story_reviews)
}
fn read_story(path: String) -> StdResult<StoryDetail, Box<dyn Error>> {
    // println!("{}",path);
    let data = fs::read_to_string(path)?;
    let story: StoryDetail = serde_json::from_str(&data)?;
    // println!("{:?}",story_review_table);
    Ok(story)
}
fn generate_stories(category: &Vec<&StoryReview>, cats: &Vec<String>) {
    for story_review in category.iter() {
        let mut prev: Option<&Story> = None;
        let mut story_list = story_review.stories.iter().peekable();
        while let Some(story) = story_list.next() {
            let next = story_list.peek().copied();
            let hello = StoryTemplate {
                story: &story,
                stories: &story_review.stories,
                activities: &category,
                activity: &story_review,
                category: &story_review.entry_type,
                prev: prev,
                next: next,
                categories: cats,
            }; // instantiate your struct
            let path_str = format!("out/{}.html", story.story_txt);
            let path = std::path::Path::new(path_str.as_str());
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();
            fs::write(path, hello.render().unwrap().replace("\\n", "<br />"))
                .expect("Unable to write file");
            prev = Some(story);
        }
    }
}
fn generate_activities(category: &Vec<&StoryReview>, cats: &Vec<String>) {
    for story_review in category.iter() {
        let hello = ActivityTemplate {
            activities: &category,
            activity: &story_review,
            category: &story_review.entry_type,
            categories: cats,
        }; // instantiate your struct
        let path_str = format!("out/act/{}.html", story_review.id);
        let path = std::path::Path::new(path_str.as_str());
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        fs::write(path, hello.render().unwrap().replace("\\n", "<br />"))
            .expect("Unable to write file");
    }
}
fn generate_category(category: &Vec<&StoryReview>, cat: String, cats: &Vec<String>) {
    let path_str = format!("out/cat/{}.html", cat);
    let hello = CategoryTemplate {
        activities: &category,
        category: &cat,
        categories: cats,
    }; // instantiate your struct
    let path = std::path::Path::new(path_str.as_str());
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    fs::write(path, hello.render().unwrap().replace("\\n", "<br />"))
        .expect("Unable to write file");
}
fn generate_index(cats: &Vec<String>) {
    let path_str = format!("out/index.html");
    let hello = IndexTemplate { categories: cats }; // instantiate your struct
    let path = std::path::Path::new(path_str.as_str());
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    fs::write(path, hello.render().unwrap().replace("\\n", "<br />"))
        .expect("Unable to write file");
}
fn main() {
    // env::set_var("RUST_BACKTRACE", "1");
    fs::remove_dir_all("out").unwrap();
    let mut story_reviews = read_story_reviews().unwrap();
    for (k, v) in story_reviews.iter_mut() {
        for activity in v {
            for story in activity.stories.iter_mut() {
                // println!("{}",story.story_txt);
                story.story_detail = read_story(format!(
                    "json/zh_CN/gamedata/story/{}.json",
                    story.story_txt
                ))
                .unwrap();
            }
        }
    }
    let cats = story_reviews.iter().map(|(k, _)| k.clone()).collect();
    generate_index(&cats);
    for (k, v) in story_reviews {
        generate_category(&v.iter().collect(), k, &cats);
        generate_activities(&v.iter().collect(), &cats);
        generate_stories(&v.iter().collect(), &cats);
    }
    fs::copy("static/leader-line.min.js", "out/leader-line.min.js").unwrap();
    fs::copy("static/style.css", "out/style.css").unwrap();
    // println!("{:?}",);
}
