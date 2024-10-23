use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Local};
use custom_debug::Debug;
use serde::{Deserialize, Serialize};
use serenity::all::Colour;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeasonName {
    Autumn,
    Spring,
    Summer,
    Winter,
}

impl Display for SeasonName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            SeasonName::Autumn => "秋",
            SeasonName::Spring => "春",
            SeasonName::Summer => "夏",
            SeasonName::Winter => "冬",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusState {
    NoState,
    OnHold,
    StopWatching,
    WannaWatch,
    Watched,
    Watching,
}

impl StatusState {
    pub fn to_colour(&self) -> Colour {
        match self {
            Self::NoState => Colour::new(0x4dd4ac),
            Self::OnHold => Colour::new(0xdc2626),
            Self::StopWatching => Colour::new(0x52525b),
            Self::WannaWatch => Colour::new(0xea580c),
            Self::Watched => Colour::new(0x16a34a),
            Self::Watching => Colour::new(0x0284c7),
        }
    }
}

impl Display for StatusState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            StatusState::NoState => "未選択",
            StatusState::OnHold => "一時中断",
            StatusState::StopWatching => "視聴中止",
            StatusState::WannaWatch => "見たい",
            StatusState::Watched => "見た",
            StatusState::Watching => "見てる",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipleRecord {
    pub records: RecordConnection,
    pub created_at: DateTime<Local>,
    pub work: Work,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordConnection {
    pub edges: Vec<RecordEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordEdge {
    pub node: Record,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub work: Work,
    pub created_at: DateTime<Local>,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub comment: Option<String>,

    pub episode: Episode,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub rating_state: Option<RatingState>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RatingState {
    Average,
    Bad,
    Good,
    Great,
}

impl RatingState {
    pub fn to_colour(&self) -> Colour {
        match self {
            Self::Average => Colour::new(0xff6d00),
            Self::Bad => Colour::new(0x757575),
            Self::Good => Colour::new(0x00c853),
            Self::Great => Colour::new(0x00b0ff),
        }
    }
}

impl Display for RatingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            RatingState::Average => "普通",
            RatingState::Bad => "良くない",
            RatingState::Good => "良い",
            RatingState::Great => "とても良い",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub number: Option<i32>,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub number_text: Option<String>,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub work: Work,
    pub created_at: DateTime<Local>,

    pub body: String,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub rating_overall_state: Option<RatingState>,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub rating_animation_state: Option<RatingState>,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub rating_character_state: Option<RatingState>,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub rating_story_state: Option<RatingState>,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub rating_music_state: Option<RatingState>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub work: Work,
    pub created_at: DateTime<Local>,
    pub state: StatusState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum ActivityItem {
    MultipleRecord(MultipleRecord),
    Record(Record),
    Review(Review),
    Status(Status),
}

impl ActivityItem {
    pub fn created_at(&self) -> DateTime<Local> {
        match self {
            Self::MultipleRecord(r) => r.created_at,
            Self::Record(r) => r.created_at,
            Self::Review(r) => r.created_at,
            Self::Status(r) => r.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityEdge {
    pub item: ActivityItem,
    pub cursor: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityConnection {
    pub edges: Vec<ActivityEdge>,
    pub page_info: PageInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub name: String,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWithActivities {
    #[serde(flatten)]
    pub user: User,

    pub activities: ActivityConnection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuery<T: std::fmt::Debug> {
    pub user: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub message: String,

    #[debug(skip_if = Option::is_none)]
    #[debug(with = opt_fmt)]
    pub locations: Option<Vec<Location>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub line: i32,
    pub column: i32,
}

/// `T` は `Data` に収容される型。
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Response<T: std::fmt::Debug> {
    Data(T),
    Errors(Vec<Error>),
}

fn opt_fmt<T: fmt::Debug>(value: &Option<T>, f: &mut Formatter) -> fmt::Result {
    write!(f, "{:?}", value.as_ref().unwrap())
}
