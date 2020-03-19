pub use mpd::{Client, Song};
use std::time::{Duration, Instant};

pub struct MpdClient {
    last_song: Option<mpd::Song>,
    last_submitted_song: Option<mpd::Song>,
    conn: mpd::Client,
}

pub struct Response {
    pub song: mpd::Song,
    pub response_type: ResponseType,
}

pub enum ResponseType {
    PlayingNow,
    Listen,
}

impl MpdClient {
    pub fn new(url: &str) -> Self {
        MpdClient {
            last_song: None,
            last_submitted_song: None,
            conn: Client::connect(url).unwrap(),
        }
    }

    pub fn poll(&mut self) -> Response {
        loop {
            let song = &self.conn.currentsong().unwrap();
            let status = &self.conn.status().unwrap();

            // println!("Tags: {:?}", song.as_ref().unwrap().tags);

            if let Some(song) = song {
                // now playing logic
                if let Some(resolved_last_song) = self.last_song.clone() {
                    if song.file != resolved_last_song.file {
                        self.last_song = Some(song.clone());
                        // submit now playing
                        return Response {
                            song: song.clone(),
                            response_type: ResponseType::PlayingNow,
                        };
                    }
                } else {
                    self.last_song = Some(song.clone());
                    // submit now playing
                    return Response {
                        song: song.clone(),
                        response_type: ResponseType::PlayingNow,
                    };
                }

                // check duration and submit scrobble
                if self.should_submit_listen(status, song) {
                    if let Some(last_submitted_song) = self.last_submitted_song.clone() {
                        if last_submitted_song.file != song.file {
                            // submit listen!
                            self.last_submitted_song = Some(song.clone());

                            return Response {
                                song: song.clone(),
                                response_type: ResponseType::Listen,
                            };
                        }
                    } else {
                        self.last_submitted_song = Some(song.clone());
                    }
                }
            }
        }
    }

    pub fn should_submit_listen(&mut self, status: &mpd::Status, song: &mpd::Song) -> bool {
        let percentage_played = self.get_song_percentage(
            status.elapsed.unwrap().to_std().unwrap(),
            song.duration.unwrap().to_std().unwrap(),
        );

        if (percentage_played >= 50.0 || status.elapsed.unwrap().num_minutes() >= 4) {
            return true;
        } else {
            return false;
        }
    }

    fn get_song_percentage(&mut self, current_time: Duration, total_time: Duration) -> f32 {
        (current_time.as_secs_f32() / total_time.as_secs_f32()) * 100.0
    }
}
