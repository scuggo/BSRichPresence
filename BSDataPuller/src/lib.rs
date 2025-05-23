pub mod livedata;
pub mod schema;
//pub mod schemaLivedata;
pub mod thread;
use crate::schema::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct BSData {
    pub gameData: Arc<Mutex<GameData>>,
    pub levelData: Arc<Mutex<LevelData>>,
    pub gamerunning: Arc<Mutex<u64>>,
}

#[derive(Debug)]
pub struct GameData {
    GameVersion: String,
    PluginVersion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelState {
    Playing,
    Paused,
    Finished,
    Failed,
    Quit,
}

#[derive(Debug, Clone)]
pub struct LevelDataInner {
    pub State: LevelState,
    pub Hash: String,
    pub SongName: String,
    pub SongSubName: String,
    pub SongAuthor: String,
    Mapper: String,
    pub CoverImage: String,
    pub RankedData: RankedData,
    pub Diff: String,
    pub Time: u32,
}

#[derive(Debug, Clone)]
pub struct RankedData {
    // Stars
    pub bl_ranked: bool,
    pub bl_qualified: bool,
    pub bl_stars: f32,
    pub ss_ranked: bool,
    pub ss_qualified: bool,
    pub ss_stars: f32,
}

pub struct RankedState {
    pub Ranked: bool,
    pub Qualified: bool,
    pub BeatleaderQualified: bool,
    pub ScoresaberQualified: bool,
    pub BeatleaderRanked: bool,
    pub ScoresaberRanked: bool,
    pub BeatleaderStars: f32,
    pub ScoresaberStars: f32,
}

#[derive(Debug)]
pub struct LevelData {
    pub LevelDataInner: Option<LevelDataInner>,
}

impl LevelData {
    pub fn overwrite_leveldata(&mut self, replacement_data: LevelDataInner) {
        self.LevelDataInner = Some(replacement_data)
    }
    pub fn update_state(&mut self, replacement_state: LevelState) {
        let build = LevelDataInner {
            State: replacement_state,
            ..self.LevelDataInner.as_ref().unwrap().clone()
        };
        self.LevelDataInner = Some(build);
    }
}

impl LevelDataInner {
    pub fn write(&mut self, awa: LevelDataInner) {
        self.SongName = awa.SongName;
        self.SongAuthor = awa.SongAuthor;
        self.CoverImage = awa.CoverImage;
    }
}

pub struct refreshBSData {
    Data: Arc<Mutex<BSData>>,
}
use reqwest::Client;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::time::sleep;
use tracing::info;
impl BSData {
    pub async fn ping() -> bool {
        // Constructs a new connection and pings and drop the connection.
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let connection = client
            .get("http://127.0.0.1:2946")
            .timeout(Duration::from_secs(5))
            .send()
            .await;
        match connection {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    pub async fn is_game_running(&self) -> bool {
        let lastMsgTimestamp = *self.gamerunning.clone().lock().await;

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        if lastMsgTimestamp < (since_the_epoch.as_secs() + 100) {
            info!(lastMsgTimestamp);
            info!("{}", since_the_epoch.as_secs() + 100);
            sleep(Duration::from_secs(1)).await;
            false
        } else {
            true
        }
    }
    //pub fn
    pub fn from_raw(data: BSMetadata) -> BSData {
        //info!(U);
        let gameData = GameData {
            GameVersion: data.GameVersion,
            PluginVersion: data.PluginVersion,
        };
        let mut levelData = LevelData {
            LevelDataInner: None,
        };
        if data.InLevel {
            levelData = LevelData {
                LevelDataInner: Some(LevelDataInner {
                    SongName: data.SongName,
                    // add the rest
                    CoverImage: {
                        // DataPuller has a habit of replying with the img as base64 and only
                        // sometimes with a usable url.
                        data.CoverImage.unwrap()
                    },
                    SongSubName: data.SongSubName,
                    SongAuthor: data.SongAuthor,
                    Hash: data.Hash.unwrap_or("0".to_owned()), // OST maps don't have a hash
                    State: {
                        if data.InLevel {
                            LevelState::Playing
                        } else {
                            LevelState::Quit
                        }
                    },
                    Time: data.Duration,
                    Mapper: data.Mapper,
                    RankedData: RankedData {
                        bl_ranked: data.RankedState.BeatleaderRanked,
                        bl_stars: data.RankedState.BeatleaderStars,
                        ss_stars: data.RankedState.ScoresaberStars,
                        ss_ranked: data.RankedState.ScoresaberRanked,
                        bl_qualified: data.RankedState.BeatleaderQualified,
                        ss_qualified: data.RankedState.ScoresaberQualified,
                    },
                    Diff: {
                        if let Some(DiffLabel) = data.CustomDifficultyLabel {
                            DiffLabel
                        } else {
                            data.Difficulty
                        }
                    },
                }),
            };
        }

        BSData {
            gameData: Arc::new(Mutex::new(gameData)),
            levelData: Arc::new(Mutex::new(levelData)),
            gamerunning: Arc::new(Mutex::new(data.UnixTimestamp)),
        }
    }
}
